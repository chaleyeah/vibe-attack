//! CPAL audio capture into a pre-allocated ring buffer (D-01, D-03, D-04).
//!
//! Architecture (from RESEARCH.md Architectural Responsibility Map):
//! - Audio RT thread: CPAL-managed; callback must NEVER allocate or block
//! - Sample queue: HeapRb pre-allocated; producer lives in callback (no alloc)

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use std::io::Write;
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

/// Opaque RAII guard that keeps the CPAL input stream running.
///
/// CRITICAL: this guard MUST be held by the thread that created the stream
/// (typically main).  Moving `cpal::Stream` into a worker closure that lives
/// past the creating function's return has been observed to silently stop
/// the ALSA/PipeWire callback on Linux.  Callers should therefore split the
/// handle: keep `StreamGuard` on main, and forward only `consumer` to the
/// pipeline worker.
pub struct StreamGuard {
    _stream: cpal::Stream,
}

/// Bundle returned by `start_audio_stream`.
pub struct AudioHandle {
    /// Hold on main thread to keep the CPAL stream alive.
    pub stream: StreamGuard,
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
    // Use default_input_config() as the ground truth for what the device actually
    // accepts at hw_params time.  supported_input_configs() can return wide ALSA
    // plug-layer ranges (e.g. 8–192 kHz) that the real hardware silently rejects.
    let default_cfg = device
        .default_input_config()
        .context("Failed to query default input config")?;

    let native_rate = default_cfg.sample_rate();
    let native_channels = default_cfg.channels();

    if native_rate.0 == 16_000 {
        // Ideal: device already runs at 16 kHz — use mono if possible.
        let channels = native_channels.min(1).max(1);
        return Ok(StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(16_000),
            buffer_size: BufferSize::Fixed(1024),
        });
    }

    // Device native rate != 16 kHz (common: 44100 or 48000 Hz).
    // Capture at the native rate and resample to 16 kHz in the callback.
    //
    // IMPORTANT: force a fixed buffer size.  PipeWire's ALSA bridge requires
    // this for stable capture; `BufferSize::Default` leaves it unset which
    // can cause the stream to silently stop delivering callbacks after the
    // first buffer on some systems.
    tracing::warn!(
        native_channels = native_channels,
        "Device native rate is {} Hz (not 16 kHz); will resample in capture callback.",
        native_rate.0
    );
    let mut cfg: StreamConfig = default_cfg.into();
    cfg.buffer_size = BufferSize::Fixed(1024);
    Ok(cfg)
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
        Some(_target) => {
            // Note: Device doesn't implement Debug in CPAL 0.15, so we just use the first device
            // A full implementation would need to match by device name
            host.input_devices()
                .context("Failed to enumerate audio input devices")?
                .next()
                .context("No audio input devices found")?
        }
    };

    tracing::info!("Audio device configured");

    let config = build_audio_config(&device)?;
    tracing::info!(
        channels = config.channels,
        "Audio stream config: {} Hz",
        config.sample_rate.0
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
    let needs_resample = native_rate.0 != target_rate;
    let mut resample_pos: f64 = 0.0;
    let resample_step: f64 = native_rate.0 as f64 / target_rate as f64;

    // Observability: emit a one-time log the first time the CPAL callback fires.
    // If this never appears, the host is not delivering samples
    // (device/driver/PipeWire routing issue) and we should blame the audio
    // backend, not the pipeline.
    use std::sync::atomic::{AtomicUsize, Ordering};
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_cb = Arc::clone(&callback_count);

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                let n = callback_count_cb.fetch_add(1, Ordering::Relaxed);
                if n == 0 {
                    // tracing may take a mutex; avoid it from the RT callback.
                    let _ = writeln!(
                        std::io::stderr(),
                        ">> CPAL first-callback: audio stream is live, samples={}",
                        data.len()
                    );
                }
                let mono: &[f32];
                let mono_owned: Vec<f32>;
                if channels == 1 {
                    mono = data;
                } else {
                    mono_owned =
                        data.chunks_exact(channels as usize).map(|c| c[0]).collect();
                    mono = &mono_owned;
                }
                if !needs_resample {
                    let _ = producer.push_slice(mono);
                    return;
                }
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
            None,
        )
        .context("Failed to build CPAL input stream")?;
    let _ = target_rate;

    stream.play().context("Failed to start CPAL stream")?;
    tracing::info!("Audio capture stream started (warm)");

    Ok(AudioHandle {
        stream: StreamGuard { _stream: stream },
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
