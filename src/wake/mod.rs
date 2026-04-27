//! Wake word keyword spotting using `sherpa-onnx`.
//!
//! Constraints:
//! - Uses only local model artifacts from config (no downloads, no network).
//! - Runs on an OS thread (never in CPAL callback, never on Tokio).
//! - Emits wake status via `tracing` (stderr). Never writes wake-related data to stdout.

use anyhow::{anyhow, Context, Result};
use std::path::Path;

use crate::config::WakeConfig;

/// Thin wrapper around sherpa-onnx keyword spotting state for one audio stream.
pub struct WakeWord {
    kws: sherpa_onnx::KeywordSpotter,
    stream: sherpa_onnx::OnlineStream,
}

impl WakeWord {
    /// Build a `WakeWord` spotter from the model paths in `cfg`.
    ///
    /// Returns an error if `wake.enabled` is false or any required model file is missing.
    /// BPE vocabulary (`bpe.model`) is detected automatically if present beside `encoder.onnx`.
    pub fn new(cfg: &WakeConfig) -> Result<Self> {
        if !cfg.enabled {
            return Err(anyhow!("wake is disabled"));
        }

        let encoder = cfg
            .encoder
            .as_ref()
            .context("wake.encoder is required when wake.enabled=true")?;
        let decoder = cfg
            .decoder
            .as_ref()
            .context("wake.decoder is required when wake.enabled=true")?;
        let joiner = cfg
            .joiner
            .as_ref()
            .context("wake.joiner is required when wake.enabled=true")?;
        let tokens = cfg
            .tokens
            .as_ref()
            .context("wake.tokens is required when wake.enabled=true")?;
        let keywords = cfg
            .keywords
            .as_ref()
            .context("wake.keywords is required when wake.enabled=true")?;

        let mut config = sherpa_onnx::KeywordSpotterConfig::default();
        config.model_config.transducer.encoder = Some(encoder.to_string_lossy().to_string());
        config.model_config.transducer.decoder = Some(decoder.to_string_lossy().to_string());
        config.model_config.transducer.joiner = Some(joiner.to_string_lossy().to_string());
        config.model_config.tokens = Some(tokens.to_string_lossy().to_string());
        config.keywords_file = Some(keywords.to_string_lossy().to_string());

        // Gigaspeech (and other BPE) KWS bundles ship `bpe.model` next to the ONNX
        // files. Sherpa requires `modeling_unit` + `bpe_vocab` for those; if the file
        // exists beside `encoder.onnx`, enable BPE automatically (no extra config key).
        let enc_path = Path::new(encoder);
        if let Some(dir) = enc_path.parent() {
            let bpe = dir.join("bpe.model");
            if bpe.is_file() {
                config.model_config.modeling_unit = Some("bpe".to_string());
                config.model_config.bpe_vocab = Some(bpe.to_string_lossy().to_string());
            }
        }

        let kws = sherpa_onnx::KeywordSpotter::create(&config)
            .ok_or_else(|| anyhow!("Failed to create sherpa-onnx KeywordSpotter"))?;
        let stream = kws.create_stream();

        Ok(Self { kws, stream })
    }

    /// Feed audio samples (16kHz mono f32) into the spotter.
    ///
    /// Returns `Some(keyword)` when a keyword is detected. Callers typically start a
    /// LISTENING window and then `reset()` the spotter state to avoid repeated triggers.
    pub fn accept_audio(&self, sample_rate_hz: i32, audio: &[f32]) {
        self.stream.accept_waveform(sample_rate_hz, audio);
    }

    /// Run decode steps until the stream is no longer ready.
    pub fn decode_until_not_ready(&self) {
        while self.kws.is_ready(&self.stream) {
            self.kws.decode(&self.stream);
        }
    }

    /// Read the current result and return the detected keyword (if any).
    pub fn take_keyword(&self) -> Option<String> {
        let res = self.kws.get_result(&self.stream)?;
        if res.keyword.trim().is_empty() {
            None
        } else {
            Some(res.keyword)
        }
    }

    /// Reset the spotter stream state so prior detections don't re-trigger on the next window.
    pub fn reset(&self) {
        self.kws.reset(&self.stream);
    }
}
