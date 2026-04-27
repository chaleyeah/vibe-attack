use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};

/// Stress test for Phase 2 concurrency / thread topology.
///
/// This is intentionally:
/// - **ignored by default** (so `cargo test` stays fast and deterministic)
/// - **env-gated** (so CI/dev machines don't accidentally run a long stress harness)
///
/// Run explicitly:
/// `RUN_STRESS_TESTS=1 cargo test --test concurrency_stress -- --include-ignored`
#[test]
#[ignore = "stress test; set RUN_STRESS_TESTS=1 to enable"]
fn concurrency_stress_pipeline_topology() {
    if std::env::var("RUN_STRESS_TESTS").ok().as_deref() != Some("1") {
        eprintln!("RUN_STRESS_TESTS != 1; skipping stress test body");
        return;
    }

    // Fixed window: must terminate in finite time.
    let duration = Duration::from_secs(2);
    let deadline = Instant::now() + duration;

    // Phase-2-like bounded queue between "VAD/utterance builder" and "STT worker".
    // Small cap to force drop-oldest behavior under load.
    let (job_tx, job_rx) = crossbeam_channel::bounded::<u64>(4);
    let job_rx_for_drop = job_rx.clone();

    // Another bounded channel representing a downstream stage (e.g., output drain).
    let (out_tx, out_rx) = crossbeam_channel::bounded::<u64>(8);
    let out_rx_for_drop = out_rx.clone();

    let stop = Arc::new(AtomicBool::new(false));
    let produced = Arc::new(AtomicU64::new(0));
    let stt_completed = Arc::new(AtomicU64::new(0));
    let drained = Arc::new(AtomicU64::new(0));

    // "Audio drain / VAD side": generate jobs quickly, never blocking.
    let t_producer = {
        let stop = Arc::clone(&stop);
        let produced = Arc::clone(&produced);
        std::thread::spawn(move || {
            let mut seq: u64 = 0;
            while !stop.load(Ordering::Relaxed) {
                seq += 1;
                produced.fetch_add(1, Ordering::Relaxed);

                // MUST NOT block: use drop-oldest helper (D-03).
                let _ = vibe_attack::vad::try_send_drop_oldest(&job_tx, &job_rx_for_drop, seq);

                // Tight loop but yield periodically to reduce busy-spin in the test harness.
                if seq.is_multiple_of(64) {
                    std::thread::yield_now();
                }
            }
        })
    };

    // "STT worker": consumes jobs slowly to simulate heavy work.
    let t_stt = {
        let stop = Arc::clone(&stop);
        let stt_completed = Arc::clone(&stt_completed);
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match job_rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(job_id) => {
                        // Simulate STT compute.
                        std::thread::sleep(Duration::from_millis(5));
                        stt_completed.fetch_add(1, Ordering::Relaxed);

                        // Produce a result downstream without blocking.
                        let _ = try_send_drop_oldest(&out_tx, &out_rx_for_drop, job_id);
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
    };

    // "Output drain": must make progress even while STT is running.
    let t_drain = {
        let stop = Arc::clone(&stop);
        let drained = Arc::clone(&drained);
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                match out_rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(_result) => {
                        drained.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
    };

    // Run for the fixed window, then stop and join.
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    stop.store(true, Ordering::Relaxed);

    t_producer.join().expect("producer thread must exit cleanly");
    t_stt.join().expect("stt thread must exit cleanly");
    t_drain.join().expect("drain thread must exit cleanly");

    let produced = produced.load(Ordering::Relaxed);
    let stt_completed = stt_completed.load(Ordering::Relaxed);
    let drained = drained.load(Ordering::Relaxed);

    assert!(produced > 0, "producer must produce jobs");
    assert!(
        stt_completed > 0,
        "STT worker must complete some jobs (produced={produced})"
    );
    assert!(
        drained > 0,
        "drain side must make progress while STT is running (stt_completed={stt_completed})"
    );
}

fn try_send_drop_oldest<T>(tx: &Sender<T>, rx: &Receiver<T>, item: T) -> Result<(), T> {
    vibe_attack::vad::try_send_drop_oldest(tx, rx, item)
}

