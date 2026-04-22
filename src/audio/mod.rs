//! CPAL audio capture with PTT gate (D-01, D-03, D-04).
//!
//! Architecture (from RESEARCH.md Architectural Responsibility Map):
//! - Audio RT thread: CPAL-managed; callback must NEVER allocate or block
//! - PTT gate: AtomicBool shared with PTT thread; Relaxed ordering sufficient
//! - Sample queue: HeapRb pre-allocated; producer lives in callback (no alloc)

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use ringbuf::{
    traits::{Producer, Split},
    HeapCons, HeapRb,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
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

/// Build the best StreamConfig for the default input device.
///
/// Prefers mono 16kHz f32. Falls back to the first supported config that covers
/// 16kHz (within min/max range), then tries stereo if mono isn't available.
/// If the device rejects 16kHz, CPAL will fail descriptively at build_input_stream.
///
/// Pitfall 1 mitigation: query supported_input_configs() before constructing manually.
pub fn build_audio_config(device: &cpal::Device) -> Result<StreamConfig> {
    let supported = device
        .supported_input_configs()
        .context("Failed to query supported input configs")?;

    // Prefer mono channel; accept stereo if mono unavailable (downsample in callback)
    let best = supported
        .filter(|c| c.min_sample_rate() <= 16_000 && c.max_sample_rate() >= 16_000)
        .min_by_key(|c| c.channels()); // prefer fewest channels (mono=1 wins)

    match best {
        Some(c) => Ok(c.with_sample_rate(16_000).into()),
        None => {
            // No config covers 16kHz — let CPAL fail at build_input_stream with details
            tracing::warn!(
                "Device does not report 16kHz support; attempting explicit StreamConfig"
            );
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
/// The callback gates samples to the ring buffer via `ptt_active` AtomicBool.
/// When PTT is not held, samples are discarded in the callback — no teardown/reinit.
///
/// # Arguments
/// * `ptt_active` — Shared flag; true when PTT is held (written by PTT thread)
///
/// # Returns
/// `AudioHandle` — must be kept alive for the stream to continue
pub fn start_audio_stream(ptt_active: Arc<AtomicBool>) -> Result<AudioHandle> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No default audio input device found")?;

    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    tracing::info!("Audio device: {}", device_name);

    let config = build_audio_config(&device)?;
    tracing::info!(
        "Audio stream config: {} ch @ {} Hz",
        config.channels,
        config.sample_rate
    );

    // Pre-allocate ring buffer — no allocation occurs in the RT callback
    let rb = HeapRb::<f32>::new(RING_BUFFER_SAMPLES);
    let (mut producer, consumer) = rb.split();

    // Extract channel count before config is moved into actual_config
    let channels = config.channels;
    let actual_config = config.clone();

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                // INVARIANT: this closure must never allocate or block.
                if ptt_active.load(Ordering::Relaxed) {
                    if channels == 1 {
                        // push_slice is a single memcpy-like call; drops silently if full
                        let _ = producer.push_slice(data);
                    } else {
                        // Stereo → mono: take left channel only (no allocation, no stack buffer)
                        for sample in data.chunks_exact(channels as usize).map(|c| c[0]) {
                            let _ = producer.try_push(sample);
                        }
                    }
                }
                // PTT not held: silently discard samples (D-04)
            },
            |err| tracing::error!("CPAL stream error: {err}"),
            None, // no timeout
        )
        .context("Failed to build CPAL input stream")?;

    stream.play().context("Failed to start CPAL stream")?;
    tracing::info!("Audio capture stream started (warm, PTT-gated)");

    Ok(AudioHandle {
        _stream: stream,
        consumer,
        actual_config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::{traits::Consumer, HeapRb};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    #[test]
    fn ptt_gate_off_discards_samples() {
        // When ptt_active is false, callback must not push anything to the ring buffer.
        let ptt = Arc::new(AtomicBool::new(false));
        let rb = HeapRb::<f32>::new(64);
        let (mut producer, mut consumer) = rb.split();

        let data = [0.1_f32; 16];
        if ptt.load(Ordering::Relaxed) {
            let _ = producer.push_slice(&data);
        }

        assert_eq!(
            consumer.pop_slice(&mut [0f32; 16]),
            0,
            "PTT off: no samples should reach the ring buffer"
        );
    }

    #[test]
    fn ptt_gate_on_pushes_samples() {
        let ptt = Arc::new(AtomicBool::new(true));
        let rb = HeapRb::<f32>::new(64);
        let (mut producer, mut consumer) = rb.split();

        let data = [0.5_f32; 8];
        if ptt.load(Ordering::Relaxed) {
            let _ = producer.push_slice(&data);
        }

        let mut out = [0f32; 8];
        let n = consumer.pop_slice(&mut out);
        assert_eq!(n, 8, "PTT on: all samples must reach ring buffer");
        assert!((out[0] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn ring_buffer_overflow_does_not_panic() {
        let rb = HeapRb::<f32>::new(4);
        let (mut producer, _consumer) = rb.split();
        // push_slice returns count pushed; when full, remaining are silently dropped
        let data = [0.1f32; 8]; // 8 samples into buffer of 4
        let _ = producer.push_slice(&data); // must not panic
    }
}
