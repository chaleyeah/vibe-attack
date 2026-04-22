//! Wake word (keyword spotter) smoke test harness (Phase 2).
//!
//! This test is skipped by default so that `cargo test -q` stays green without
//! local model artifacts present.
//!
//! Run with:
//! `RUN_KWS_TESTS=1 KWS_ENCODER=... KWS_DECODER=... KWS_JOINER=... KWS_TOKENS=... KWS_KEYWORDS=... cargo test --test wake_word -- --include-ignored`

use std::path::Path;

fn should_run_kws() -> bool {
    std::env::var("RUN_KWS_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn env_path(name: &str) -> String {
    let v = std::env::var(name).unwrap_or_else(|_| panic!("Missing env var {name}"));
    assert!(Path::new(&v).exists(), "{name} does not exist: {v}");
    v
}

#[test]
#[ignore = "heavy wake-word test — set RUN_KWS_TESTS=1 and KWS_* paths"]
fn keyword_spotter_loads_and_decodes_silence() {
    if !should_run_kws() {
        return;
    }

    let encoder = env_path("KWS_ENCODER");
    let decoder = env_path("KWS_DECODER");
    let joiner = env_path("KWS_JOINER");
    let tokens = env_path("KWS_TOKENS");
    let keywords = env_path("KWS_KEYWORDS");

    let mut config = sherpa_onnx::KeywordSpotterConfig::default();
    config.model_config.transducer.encoder = Some(encoder);
    config.model_config.transducer.decoder = Some(decoder);
    config.model_config.transducer.joiner = Some(joiner);
    config.model_config.tokens = Some(tokens);
    config.keywords_file = Some(keywords);

    let kws = sherpa_onnx::KeywordSpotter::create(&config)
        .expect("create keyword spotter (check model paths)");
    let stream = kws.create_stream();

    // 1 second of silence @ 16kHz. We just want to validate the decode loop doesn't crash.
    let audio = vec![0.0f32; 16_000];
    stream.accept_waveform(16_000, &audio);
    stream.input_finished();

    while kws.is_ready(&stream) {
        kws.decode(&stream);
    }

    // It's fine if there's no trigger; we only care that the decode path works.
    let _ = kws.get_result(&stream);
    let _ = kws.get_result_as_json(&stream);
}

