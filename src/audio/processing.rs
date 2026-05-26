use rubato::{Fft, Indexing, Resampler, audioadapter_buffers::direct::InterleavedSlice};

use crate::errors::AppError;

pub const CHUNK_SIZE: usize = 1024;

pub struct Processor {
    resampler: Option<Fft<f32>>,

    resample_buf: Vec<f32>,
    channel_buf: Vec<f32>,

    input_channels: usize,
    output_channels: usize,
}

impl Processor {
    pub fn new(
        input_channels: u16,
        output_channels: u16,
        input_sample_rate: u32,
        output_sample_rate: u32,
    ) -> Result<Self, AppError> {
        let input_channels = input_channels as usize;
        let output_channels = output_channels as usize;

        let resampler = if input_sample_rate != output_sample_rate {
            Some(
                Fft::<f32>::new(
                    input_sample_rate as usize,
                    output_sample_rate as usize,
                    CHUNK_SIZE,
                    2,
                    input_channels,
                    rubato::FixedSync::Input,
                )
                .map_err(|e| {
                    log::warn!("Failed to create resampler: {e}");
                    AppError::AudioStreamFailed
                })?,
            )
        } else {
            None
        };

        let max_frames = resampler
            .as_ref()
            .map(|r| r.output_frames_max())
            .unwrap_or(CHUNK_SIZE);

        Ok(Self {
            resample_buf: vec![0.0; max_frames * input_channels],
            channel_buf: vec![0.0; max_frames * output_channels],
            resampler,
            input_channels,
            output_channels,
        })
    }

    pub fn needs_fixed_chunks(&self) -> bool {
        self.resampler.is_some()
    }

    pub fn process_chunk(&mut self, input: &[f32], volume: f32) -> &[f32] {
        let input_frames = input.len() / self.input_channels;
        let in_channels = self.input_channels;
        let out_channels = self.output_channels;

        // Stage 1: Resample
        let (resampled_data, frame_count) = if let Some(ref mut r) = self.resampler {
            let output_frames = r.output_frames_next();

            let input_adapter = InterleavedSlice::new(input, in_channels, input_frames).unwrap();
            let mut output_adapter =
                InterleavedSlice::new_mut(&mut self.resample_buf, in_channels, output_frames)
                    .unwrap();

            let indexing = Indexing {
                input_offset: 0,
                output_offset: 0,
                partial_len: None,
                active_channels_mask: None,
            };

            let (_, frames_written) =
                match r.process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing)) {
                    Ok(result) => result,
                    Err(e) => {
                        log::warn!("Resampling failed: {e}");
                        return &self.channel_buf[..0];
                    }
                };

            (
                &self.resample_buf[..frames_written * in_channels],
                frames_written,
            )
        } else {
            (input, input_frames)
        };

        // Stage 2: Channel conversion
        if in_channels == out_channels {
            let len = frame_count * in_channels;
            self.channel_buf[..len].copy_from_slice(&resampled_data[..len]);
            return &self.channel_buf[..len];
        }

        for frame in 0..frame_count {
            let in_start = frame * in_channels;
            let out_start = frame * out_channels;

            for out_ch in 0..out_channels {
                let sample = match (in_channels, out_channels) {
                    (2, 1) => (resampled_data[in_start] + resampled_data[in_start + 1]) * 0.5,
                    _ => resampled_data[in_start + (out_ch % in_channels)],
                };
                self.channel_buf[out_start + out_ch] = sample;
            }
        }

        for sample in &mut self.channel_buf[..frame_count * out_channels] {
            *sample *= volume;
        }

        &self.channel_buf[..frame_count * out_channels]
    }
}
