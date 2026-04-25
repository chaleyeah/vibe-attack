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
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .expect("Set WHISPER_MODEL_PATH to a local whisper.cpp model file (e.g. tiny.en.bin)");
        assert!(
            std::path::Path::new(&model_path).exists(),
            "Model path does not exist: {model_path}"
        );

        use tokio_util::sync::CancellationToken;

        let shutdown = CancellationToken::new();
        let stt = vibe_attack::stt::SttService::new(&model_path, shutdown.clone())
            .expect("create STT service");
        let mut stt = stt.spawn().expect("spawn STT thread");

        // 1 second of silence @ 16kHz. We just want to validate the end-to-end call path.
        let job = vibe_attack::vad::UtteranceJob {
            utterance_id: 1,
            audio: vec![0.0f32; 16_000],
            timings: vibe_attack::pipeline::timing::UtteranceTimings::new(),
            start_frame_idx: 0,
            end_frame_idx: 0,
        };

        stt.try_submit(job).expect("submit utterance job");

        let result = stt
            .result_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("receive STT result");
        let _ = result.text; // may be empty on silence; that's fine

        shutdown.cancel();
        stt.request_shutdown();
        stt.join_best_effort(std::time::Duration::from_millis(500));
    }
}

