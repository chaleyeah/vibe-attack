//! STT smoke test harness (Phase 2).
//!
//! This is intentionally heavy and is skipped by default so that `cargo test -q`
//! stays green on machines without local models/tooling.
//!
//! Run with:
//! `RUN_STT_TESTS=1 WHISPER_MODEL_PATH=/path/to/tiny.en.bin cargo test --test stt_smoke -- --include-ignored`

fn should_run_stt() -> bool {
    std::env::var("RUN_STT_TESTS")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[test]
#[ignore = "heavy STT test — set RUN_STT_TESTS=1 and WHISPER_MODEL_PATH"]
fn whisper_loads_model_and_runs_one_pass() {
    if !should_run_stt() {
        return;
    }

    #[cfg(not(feature = "stt"))]
    {
        panic!("This test requires building with `--features stt` (enables whisper-rs).");
    }

    #[cfg(feature = "stt")]
    {
        use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .expect("Set WHISPER_MODEL_PATH to a local whisper.cpp model file (e.g. tiny.en.bin)");
        assert!(
            std::path::Path::new(&model_path).exists(),
            "Model path does not exist: {model_path}"
        );

        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default(),
        )
        .expect("load whisper model");

        let mut state = ctx.create_state().expect("create whisper state");
        let params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // 1 second of silence @ 16kHz. We just want to validate the call path.
        let audio = vec![0.0f32; 16_000];
        state.full(params, &audio).expect("run whisper full()");
    }
}

