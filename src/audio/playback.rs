use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
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

#[allow(dead_code)]
pub struct AudioStreams {
    pub input_stream: Stream,
    pub output_stream: Stream,
}

impl AudioStreams {
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
                    if pushed_any && let Ok(mut ready) = wake_pair_clone.0.try_lock() {
                        *ready = true;
                        wake_pair_clone.1.notify_one();
                    }
                },
                |err| log::error!("Input error: {err}"),
                None,
            )
            .map_err(|e| {
                log::warn!("Failed to build input stream: {e}");
                AppError::AudioStreamFailed
            })?;

        let mut processor = Processor::new(
            stream_config.input_channels,
            stream_config.output_channels,
            stream_config.input_sample_rate,
            stream_config.output_sample_rate,
        )?;

        thread::spawn(move || {
            let samples_per_chunk = CHUNK_SIZE * input_channels;
            let mut accumulator: Vec<f32> = Vec::with_capacity(samples_per_chunk);
            let (lock, cvar) = &*wake_pair;

            loop {
                let overruns = input_overruns.swap(0, Ordering::Relaxed);
                if overruns > 0 {
                    log::warn!("Input overrun! Dropped {overruns} samples.");
                }

                while let Some(sample) = raw_consumer.try_pop() {
                    accumulator.push(sample);
                }

                if processor.needs_fixed_chunks() {
                    while accumulator.len() >= samples_per_chunk {
                        for &sample in processor.process_chunk(
                            &accumulator[..samples_per_chunk],
                            *device_config.volume.lock().unwrap(),
                        ) {
                            processed_producer.try_push(sample).ok();
                        }
                        accumulator.drain(..samples_per_chunk);
                    }
                } else if !accumulator.is_empty() {
                    for &sample in
                        processor.process_chunk(&accumulator, *device_config.volume.lock().unwrap())
                    {
                        processed_producer.try_push(sample).ok();
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
                |err| log::error!("Output error: {err}"),
                None,
            )
            .map_err(|e| {
                log::warn!("Failed to build output stream: {e}");
                AppError::AudioStreamFailed
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
        })
    }
}
