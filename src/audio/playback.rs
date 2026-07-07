use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use cpal::{
    Stream,
    traits::{DeviceTrait, StreamTrait},
};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Producer, Split},
};

use crate::{
    audio::{
        config::{AudioConfig, NegotiatedConfig},
        io::{find_audio_input, find_audio_output},
        processing::{CHUNK_SIZE, Processor},
    },
    errors::AppError,
};

const STALE_THRESHOLD: Duration = Duration::from_secs(3);

#[allow(dead_code)]
pub struct AudioStreams {
    input_stream: Stream,
    output_stream: Stream,
    process_stop: Arc<AtomicBool>,
    input_errored: Arc<AtomicBool>,
    output_errored: Arc<AtomicBool>,
    processing_errored: Arc<AtomicBool>,
    activity_base: Instant,
    last_activity_ms: Arc<AtomicU64>,
}

impl Drop for AudioStreams {
    fn drop(&mut self) {
        self.process_stop.store(true, Ordering::Relaxed);
    }
}

impl AudioStreams {
    pub fn has_failed(&self) -> bool {
        if self.input_errored.load(Ordering::Relaxed)
            || self.output_errored.load(Ordering::Relaxed)
            || self.processing_errored.load(Ordering::Relaxed)
        {
            return true;
        }
        let elapsed_ms = self.activity_base.elapsed().as_millis() as u64;
        let last_ms = self.last_activity_ms.load(Ordering::Relaxed);
        elapsed_ms.saturating_sub(last_ms) > STALE_THRESHOLD.as_millis() as u64
    }

    pub fn start_playback(device_config: AudioConfig) -> Result<Self, AppError> {
        let host = cpal::default_host();

        let input_device = find_audio_input(&device_config.input_device, &host)?;
        let output_device = find_audio_output(&device_config.output_device, &host)?;

        let stream_config = NegotiatedConfig::negotiate_configs(&input_device, &output_device)?;
        let input_channels = stream_config.input_channels as usize;
        let output_channels = stream_config.output_channels as usize;

        let (mut raw_producer, mut raw_consumer) =
            HeapRb::<f32>::new(CHUNK_SIZE * input_channels * 8).split();

        let (mut processed_producer, mut processed_consumer) =
            HeapRb::<f32>::new(CHUNK_SIZE * output_channels * 8).split();

        let input_overruns = Arc::new(AtomicUsize::new(0));
        let input_overruns_clone = input_overruns.clone();

        let wake_pair = Arc::new((Mutex::new(false), Condvar::new()));
        let wake_pair_clone = wake_pair.clone();

        let activity_base = Instant::now();
        let last_activity_ms = Arc::new(AtomicU64::new(0));
        let last_activity_ms_clone = last_activity_ms.clone();

        let input_errored = Arc::new(AtomicBool::new(false));
        let input_errored_clone = input_errored.clone();
        let output_errored = Arc::new(AtomicBool::new(false));
        let output_errored_clone = output_errored.clone();
        let processing_errored = Arc::new(AtomicBool::new(false));
        let processing_errored_clone = processing_errored.clone();

        let input_stream = input_device
            .build_input_stream::<f32, _, _>(
                &stream_config.input_config,
                move |data: &[f32], _| {
                    let mut pushed_any = false;
                    for &sample in data {
                        if raw_producer.try_push(sample).is_err() {
                            input_overruns_clone.fetch_add(1, Ordering::Relaxed);
                        } else {
                            pushed_any = true;
                        }
                    }

                    if !data.is_empty() {
                        last_activity_ms_clone.store(
                            activity_base.elapsed().as_millis() as u64,
                            Ordering::Relaxed,
                        );
                    }
                    if pushed_any && let Ok(mut ready) = wake_pair_clone.0.try_lock() {
                        *ready = true;
                        wake_pair_clone.1.notify_one();
                    }
                },
                move |err| {
                    log::error!("Input stream error: {err}");
                    input_errored_clone.store(true, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|e| {
                log::warn!("Failed to build input stream: {e}");
                match e {
                    cpal::BuildStreamError::DeviceNotAvailable => AppError::AudioDeviceInUse,
                    _ => AppError::AudioStreamFailed,
                }
            })?;

        let mut processor = Processor::new(
            stream_config.input_channels,
            stream_config.output_channels,
            stream_config.input_sample_rate,
            stream_config.output_sample_rate,
        )?;

        let process_stop = Arc::new(AtomicBool::new(false));
        let process_stop_clone = Arc::clone(&process_stop);

        thread::spawn(move || {
            let samples_per_chunk = CHUNK_SIZE * input_channels;
            let mut accumulator: Vec<f32> = Vec::with_capacity(samples_per_chunk);
            let (lock, cvar) = &*wake_pair;

            loop {
                if process_stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                let overruns = input_overruns.swap(0, Ordering::Relaxed);
                if overruns > 0 {
                    log::warn!("Input overrun! Dropped {overruns} samples.");
                }

                while let Some(sample) = raw_consumer.try_pop() {
                    accumulator.push(sample);
                }

                if processor.needs_fixed_chunks() {
                    while accumulator.len() >= samples_per_chunk {
                        match processor.process_chunk(
                            &accumulator[..samples_per_chunk],
                            *device_config.volume.lock().unwrap(),
                        ) {
                            Some(out) => {
                                for &sample in out {
                                    processed_producer.try_push(sample).ok();
                                }
                            }
                            None => processing_errored_clone.store(true, Ordering::Relaxed),
                        }
                        accumulator.drain(..samples_per_chunk);
                    }
                } else if !accumulator.is_empty() {
                    match processor
                        .process_chunk(&accumulator, *device_config.volume.lock().unwrap())
                    {
                        Some(out) => {
                            for &sample in out {
                                processed_producer.try_push(sample).ok();
                            }
                        }
                        None => processing_errored_clone.store(true, Ordering::Relaxed),
                    }
                    accumulator.clear();
                }

                let mut ready = lock.lock().unwrap();
                if !*ready && accumulator.len() < samples_per_chunk {
                    ready = cvar
                        .wait_timeout(ready, Duration::from_millis(5))
                        .unwrap()
                        .0;
                }
                *ready = false;
            }
        });

        let output_stream = output_device
            .build_output_stream::<f32, _, _>(
                &stream_config.output_config,
                move |data: &mut [f32], _| {
                    for sample in data {
                        *sample = processed_consumer.try_pop().unwrap_or(0.0);
                    }
                },
                move |err| {
                    log::error!("Output stream error: {err}");
                    output_errored_clone.store(true, Ordering::Relaxed);
                },
                None,
            )
            .map_err(|e| {
                log::warn!("Failed to build output stream: {e}");
                match e {
                    cpal::BuildStreamError::DeviceNotAvailable => AppError::AudioDeviceInUse,
                    _ => AppError::AudioStreamFailed,
                }
            })?;

        input_stream.play().map_err(|e| {
            log::warn!("Failed to start input stream: {e}");
            AppError::AudioStreamFailed
        })?;

        output_stream.play().map_err(|e| {
            log::warn!("Failed to start output stream: {e}");
            AppError::AudioStreamFailed
        })?;

        Ok(Self {
            input_stream,
            output_stream,
            process_stop,
            input_errored,
            output_errored,
            processing_errored, // new
            activity_base,
            last_activity_ms,
        })
    }
}

pub struct AudioSupervisor {
    current: Arc<Mutex<Option<AudioStreams>>>,
    stop: Arc<AtomicBool>,
    watchdog: Option<JoinHandle<()>>,
    notice: Arc<Mutex<Option<String>>>,
}

impl AudioSupervisor {
    pub fn start(config: AudioConfig) -> Result<Self, AppError> {
        let streams = AudioStreams::start_playback(config.clone())?;
        let current = Arc::new(Mutex::new(Some(streams)));
        let stop = Arc::new(AtomicBool::new(false));
        let notice: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let current_clone = current.clone();
        let stop_clone = stop.clone();
        let notice_clone = notice.clone();

        let watchdog = thread::spawn(move || {
            let mut backoff = Duration::from_secs(1);
            const MAX_BACKOFF: Duration = Duration::from_secs(10);

            loop {
                thread::sleep(Duration::from_millis(500));
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                let needs_restart = match &*current_clone.lock().unwrap() {
                    None => true,
                    Some(streams) => streams.has_failed(),
                };

                if !needs_restart {
                    backoff = Duration::from_secs(1);
                    continue;
                }

                log::warn!(
                    "Audio stream failed or went stale (capture card audio glitch?); \
                     attempting to reconnect without a replug..."
                );

                *current_clone.lock().unwrap() = None;

                match AudioStreams::start_playback(config.clone()) {
                    Ok(new_streams) => {
                        *current_clone.lock().unwrap() = Some(new_streams);
                        log::info!("Audio stream reconnected automatically.");
                        *notice_clone.lock().unwrap() =
                            Some("Audio reconnected automatically.".to_string());
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        log::warn!("Automatic audio reconnect failed: {e}. Will retry.");
                        backoff = (backoff * 2).min(MAX_BACKOFF);
                        thread::sleep(backoff);
                    }
                }
            }
        });

        Ok(Self {
            current,
            stop,
            watchdog: Some(watchdog),
            notice,
        })
    }

    pub fn take_notice(&self) -> Option<String> {
        self.notice.lock().unwrap().take()
    }
}

impl Drop for AudioSupervisor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);

        *self.current.lock().unwrap() = None;

        self.watchdog.take();
    }
}
