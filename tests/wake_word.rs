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

#[test]
#[ignore = "dual-ORT coexistence test — set RUN_KWS_TESTS=1 and KWS_* paths; requires libonnxruntime.so in target/debug/"]
fn dual_init_wake_and_vad_coexist() {
    if !should_run_kws() {
        return;
    }

    // Mirror the coordinator's ORT_DYLIB_PATH auto-discovery: point both ORT consumers
    // at the same shared .so so they share one schema registry and allocator.
    if std::env::var("ORT_DYLIB_PATH").is_err() {
        let so_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/debug/libonnxruntime.so");
        // Safety: single-threaded at this point — no other threads have spawned.
        unsafe { std::env::set_var("ORT_DYLIB_PATH", &so_path) };
    }

    // --- sherpa-onnx KeywordSpotter init ---
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

    let audio = vec![0.0f32; 16_000];
    stream.accept_waveform(16_000, &audio);
    stream.input_finished();
    while kws.is_ready(&stream) {
        kws.decode(&stream);
    }
    let _ = kws.get_result(&stream);

    // --- silero-vad-rust OnnxModel init (second ORT consumer in the same process) ---
    // If the dual-ORT conflict is resolved, this must not panic or bad_alloc.
    let _vad = silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
        silero_vad_rust::silero_vad::model::LoadOptions {
            force_onnx_cpu: true,
            ..Default::default()
        },
    )
    .expect("load silero VAD after sherpa-onnx — dual-ORT coexistence must hold");
}

