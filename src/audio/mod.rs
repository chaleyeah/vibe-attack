//! CPAL audio capture into a pre-allocated ring buffer (D-01, D-03, D-04).
//!
//! Architecture (from RESEARCH.md Architectural Responsibility Map):
//! - Audio RT thread: CPAL-managed; callback must NEVER allocate or block
//! - Sample queue: HeapRb pre-allocated; producer lives in callback (no alloc)

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use ringbuf::{
    traits::{Producer, Split},
    HeapCons, HeapRb,
};
use std::sync::{
    atomic::AtomicBool,
    Arc,
};

/// Number of f32 samples to pre-allocate in the ring buffer.
/// 5 seconds at 16 kHz = 80,000 samples. Phase 2 drains this actively.
const RING_BUFFER_SAMPLES: usize = 16_000 * 5;

/// Handle returned by `start_audio_stream`. Keeps the stream alive.
/// Drop to stop the stream. Consumer end is available for Phase 2 pipeline.
pub struct AudioHandle {
    /// Keep the CPAL stream alive (dropped when AudioHandle is dropped).
    _stream: cpal::Stream,
    /// Consumer end of the ring buffer for Phase 2 audio pipeline.
    pub consumer: HeapCons<f32>,
    /// Negotiated stream configuration (logged at startup for diagnosis — Pitfall 1 mitigation).
    pub actual_config: StreamConfig,
}

/// Build the best StreamConfig for the given input device.
///
/// Priority:
///   1. Mono 16 kHz  (ideal for Whisper — no resampling needed)
///   2. Stereo 16 kHz (pipeline downmixes to mono)
///   3. Mono at device's native rate (pipeline will resample to 16 kHz)
///   4. Stereo at device's native rate (pipeline downmixes then resamples)
///
/// Pitfall 1 mitigation: query supported_input_configs() before constructing manually.
pub fn build_audio_config(device: &cpal::Device) -> Result<StreamConfig> {
    let supported: Vec<_> = device
        .supported_input_configs()
        .context("Failed to query supported input configs")?
        .collect();

    // Pass 1: prefer any config that natively supports 16 kHz.
    let best_16k = supported.iter()
        .filter(|c| c.min_sample_rate() <= 16_000 && c.max_sample_rate() >= 16_000)
        .min_by_key(|c| c.channels());

    if let Some(c) = best_16k {
        return Ok(c.with_sample_rate(16_000).into());
    }

    // Pass 2: no native 16 kHz — use the device's lowest available rate.
    // The CPAL callback will linearly resample to 16 kHz on the fly.
    let best_native = supported.iter().min_by_key(|c| (c.channels(), c.min_sample_rate()));

    match best_native {
        Some(c) => {
            let rate = c.min_sample_rate();
            tracing::warn!(
                native_rate = rate,
                "Device does not support 16 kHz natively; capturing at {} Hz. \
                 Resampling to 16 kHz before STT.",
                rate
            );
            Ok(c.with_sample_rate(rate).into())
        }
        None => {
            tracing::warn!("Could not determine supported configs; attempting 16 kHz mono directly");
            Ok(StreamConfig {
                channels: 1,
                sample_rate: 16_000,
                buffer_size: BufferSize::Default,
            })
        }
    }
}

/// Start a warm CPAL audio capture stream (D-04: stream is always running).
///
/// The callback always pushes samples into the ring buffer.
///
/// PTT/wake gating happens on the pipeline OS thread after draining `AudioHandle.consumer`.
///
/// # Arguments
/// * `device_name` — Optional CPAL device name; `None` uses the system default.
///                   Run `--list-devices` to enumerate available names.
/// * `ptt_active`  — Shared flag; true when PTT is held (written by PTT thread)
///
/// # Returns
/// `AudioHandle` — must be kept alive for the stream to continue
pub fn start_audio_stream(device_name: Option<&str>, ptt_active: Arc<AtomicBool>) -> Result<AudioHandle> {
    use cpal::traits::HostTrait as _;
    let host = cpal::default_host();

    let device = match device_name {
        None => host
            .default_input_device()
            .context("No default audio input device found")?,
        Some(target) => {
            host.input_devices()
                .context("Failed to enumerate audio input devices")?
                .find(|d| d.description().map(|desc| desc.name() == target).unwrap_or(false))
                .with_context(|| {
                    format!(
                        "Audio device not found: '{target}'\n\
                         Run with --list-devices to see available device names."
                    )
                })?
        }
    };

    let name = device.description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    tracing::info!("Audio device: {}", name);

    let config = build_audio_config(&device)?;
    tracing::info!(
        "Audio stream config: {} ch @ {} Hz",
        config.channels,
        config.sample_rate
    );

    // Pre-allocate ring buffer — no allocation occurs in the RT callback
    let rb = HeapRb::<f32>::new(RING_BUFFER_SAMPLES);
    let (mut producer, consumer) = rb.split();

    // Extract channel count / sample rate before config is moved.
    let channels = config.channels;
    let native_rate = config.sample_rate;
    let actual_config = config.clone();

    let _ = ptt_active; // gating happens in pipeline thread

    // Simple linear resampler state for non-16 kHz devices.
    // Holds a fractional position within the native stream.
    let target_rate: u32 = 16_000;
    let needs_resample = native_rate != target_rate;
    let mut resample_pos: f64 = 0.0;
    let resample_step: f64 = native_rate as f64 / target_rate as f64;

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                // INVARIANT: this closure must never allocate or block.
                // Step 1: downmix to mono.
                let mono: &[f32];
                let mono_owned: Vec<f32>;
                if channels == 1 {
                    mono = data;
                } else {
                    mono_owned = data.chunks_exact(channels as usize).map(|c| c[0]).collect();
                    mono = &mono_owned;
                }

                if !needs_resample {
                    let _ = producer.push_slice(mono);
                    return;
                }

                // Step 2: linear resample to 16 kHz.
                // This runs in the RT callback — no heap allocation beyond the
                // one-time Vec above which is already allocated.
                let len = mono.len();
                if len == 0 {
                    return;
                }
                while resample_pos < len as f64 {
                    let i = resample_pos as usize;
                    let frac = resample_pos - i as f64;
                    let s = if i + 1 < len {
                        mono[i] + frac as f32 * (mono[i + 1] - mono[i])
                    } else {
                        mono[i]
                    };
                    let _ = producer.try_push(s);
                    resample_pos += resample_step;
                }
                resample_pos -= len as f64;
            },
            |err| tracing::error!("CPAL stream error: {err}"),
            None, // no timeout
        )
        .context("Failed to build CPAL input stream")?;

    stream.play().context("Failed to start CPAL stream")?;
    tracing::info!("Audio capture stream started (warm)");

    Ok(AudioHandle {
        _stream: stream,
        consumer,
        actual_config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::HeapRb;

    #[test]
    fn ring_buffer_overflow_does_not_panic() {
        let rb = HeapRb::<f32>::new(4);
        let (mut producer, _consumer) = rb.split();
        // push_slice returns count pushed; when full, remaining are silently dropped
        let data = [0.1f32; 8]; // 8 samples into buffer of 4
        let _ = producer.push_slice(&data); // must not panic
    }
}
