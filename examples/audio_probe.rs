//! Standalone CPAL audio probe.
//!
//! Captures from a device (or default) for N seconds and reports how many
//! samples CPAL actually delivered.  Run this when the pipeline heartbeat
//! shows `samples=0` to isolate whether the issue is CPAL/audio-backend or
//! the pipeline itself.
//!
//! Usage:
//!   cargo run --release --example audio_probe -- --list
//!   cargo run --release --example audio_probe -- --device "PipeWire Sound Server" --secs 5
//!   cargo run --release --example audio_probe -- --device "PulseAudio Sound Server"
//!   cargo run --release --example audio_probe                 # default device

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Toggle tokio by setting --with-tokio on the CLI.  Tokio init happens in a
// separate helper that spawns main-equivalent under the runtime.
fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--via-lib-tokio") {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        return rt.block_on(async { via_lib_flow() });
    }
    if std::env::args().any(|a| a == "--audio-before-runtime") {
        // Start audio + pipeline BEFORE the tokio runtime is created.
        via_lib_flow_no_block()?;
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        return rt.block_on(async {
            eprintln!(">> entering tokio::select! for 6s (audio started pre-runtime)");
            tokio::time::sleep(Duration::from_secs(6)).await;
            Ok(())
        });
    }
    if std::env::args().any(|a| a == "--no-tokio-wait") {
        via_lib_flow_no_block()?;
        eprintln!(">> sleeping 6s via std::thread::sleep (no tokio)");
        std::thread::sleep(Duration::from_secs(6));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--inline-nested-block") {
        use ringbuf::traits::Consumer;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SAMPLES: AtomicUsize = AtomicUsize::new(0);
        let keep_ptt = std::env::args().any(|a| a == "--keep-ptt");
        let outer_ptt;
        {
            let device_name: Option<String> = {
                let mut it = std::env::args();
                let mut out = None;
                while let Some(a) = it.next() {
                    if a == "--device" { out = it.next(); }
                }
                out
            };
            let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            if keep_ptt { outer_ptt = Some(ptt.clone()); } else { outer_ptt = None; }
            let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt)?;
            eprintln!(">> (nested) stream started; spawning pipeline thread");
            let join = std::thread::spawn(move || {
                eprintln!(">> (pipeline-bg) thread entered; polling");
                let mut cons = handle.consumer;
                let mut buf = [0f32; 1024];
                let start = Instant::now();
                let mut last = Instant::now();
                while start.elapsed() < Duration::from_secs(5) {
                    let n = cons.pop_slice(&mut buf);
                    if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
                    else { SAMPLES.fetch_add(n, Ordering::Relaxed); }
                    if last.elapsed() >= Duration::from_secs(1) {
                        eprintln!(">> (pipeline-bg) @{}s: samples={}",
                                  start.elapsed().as_secs(),
                                  SAMPLES.load(Ordering::Relaxed));
                        last = Instant::now();
                    }
                }
                eprintln!(">> (pipeline-bg) total samples drained: {}",
                          SAMPLES.load(Ordering::Relaxed));
            });
            Box::leak(Box::new(join));
        }
        eprintln!(">> main: sleeping 6s AFTER nested block ended (keep_ptt={keep_ptt})");
        std::thread::sleep(Duration::from_secs(6));
        drop(outer_ptt);
        return Ok(());
    }
    if std::env::args().any(|a| a == "--audio-in-block-spawn-out") {
        // start_audio_stream inside a nested block; spawn pipeline OUTSIDE.
        // If the block boundary is what's breaking audio, this should still
        // fail.  If it's something about spawn, this will work.
        use ringbuf::traits::Consumer;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SAMPLES: AtomicUsize = AtomicUsize::new(0);
        let device_name: Option<String> = {
            let mut it = std::env::args();
            let mut out = None;
            while let Some(a) = it.next() {
                if a == "--device" { out = it.next(); }
            }
            out
        };
        let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = {
            let h = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt.clone())?;
            eprintln!(">> audio started inside block");
            h
        };
        eprintln!(">> block exited; now spawning pipeline thread");
        let _join = std::thread::spawn(move || {
            let mut cons = handle.consumer;
            let mut buf = [0f32; 1024];
            let start = Instant::now();
            let mut last = Instant::now();
            while start.elapsed() < Duration::from_secs(5) {
                let n = cons.pop_slice(&mut buf);
                if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
                else { SAMPLES.fetch_add(n, Ordering::Relaxed); }
                if last.elapsed() >= Duration::from_secs(1) {
                    eprintln!(">> @{}s: samples={}",
                              start.elapsed().as_secs(),
                              SAMPLES.load(Ordering::Relaxed));
                    last = Instant::now();
                }
            }
            eprintln!(">> total: {}", SAMPLES.load(Ordering::Relaxed));
        });
        std::thread::sleep(Duration::from_secs(6));
        let _ = device_name;
        let _ = ptt;
        return Ok(());
    }
    if std::env::args().any(|a| a == "--inline-leak-block-minimal") {
        // Minimal nested-block repro: put ONLY start_audio_stream + spawn in
        // the inner block.  Everything else (device_name, ptt) stays outside.
        use ringbuf::traits::Consumer;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SAMPLES: AtomicUsize = AtomicUsize::new(0);
        let device_name: Option<String> = {
            let mut it = std::env::args();
            let mut out = None;
            while let Some(a) = it.next() {
                if a == "--device" { out = it.next(); }
            }
            out
        };
        let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt.clone())?;
            eprintln!(">> (minimal-block) stream started");
            let join = std::thread::spawn(move || {
                let mut cons = handle.consumer;
                let mut buf = [0f32; 1024];
                let start = Instant::now();
                let mut last = Instant::now();
                while start.elapsed() < Duration::from_secs(5) {
                    let n = cons.pop_slice(&mut buf);
                    if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
                    else { SAMPLES.fetch_add(n, Ordering::Relaxed); }
                    if last.elapsed() >= Duration::from_secs(1) {
                        eprintln!(">> @{}s: samples={}",
                                  start.elapsed().as_secs(),
                                  SAMPLES.load(Ordering::Relaxed));
                        last = Instant::now();
                    }
                }
                eprintln!(">> total: {}", SAMPLES.load(Ordering::Relaxed));
            });
            Box::leak(Box::new(join));
        }
        eprintln!(">> main: sleeping 6s after minimal inner block");
        std::thread::sleep(Duration::from_secs(6));
        let _ = device_name;
        let _ = ptt;
        return Ok(());
    }
    if std::env::args().any(|a| a == "--inline-leak-block") {
        use ringbuf::traits::Consumer;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SAMPLES: AtomicUsize = AtomicUsize::new(0);
        {
            let device_name: Option<String> = {
                let mut it = std::env::args();
                let mut out = None;
                while let Some(a) = it.next() {
                    if a == "--device" { out = it.next(); }
                }
                out
            };
            let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt)?;
            eprintln!(">> (inline-leak-block) stream started");
            let join = std::thread::spawn(move || {
                let mut cons = handle.consumer;
                let mut buf = [0f32; 1024];
                let start = Instant::now();
                let mut last = Instant::now();
                while start.elapsed() < Duration::from_secs(5) {
                    let n = cons.pop_slice(&mut buf);
                    if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
                    else { SAMPLES.fetch_add(n, Ordering::Relaxed); }
                    if last.elapsed() >= Duration::from_secs(1) {
                        eprintln!(">> @{}s: samples={}",
                                  start.elapsed().as_secs(),
                                  SAMPLES.load(Ordering::Relaxed));
                        last = Instant::now();
                    }
                }
                eprintln!(">> total: {}", SAMPLES.load(Ordering::Relaxed));
            });
            Box::leak(Box::new(join));
        }
        eprintln!(">> main: sleeping 6s after inner block end");
        std::thread::sleep(Duration::from_secs(6));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--inline-leak") {
        // Same as --inline-no-block but Box::leak the JoinHandle instead
        // of binding to _join.  Check if that alone breaks audio.
        use ringbuf::traits::Consumer;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SAMPLES: AtomicUsize = AtomicUsize::new(0);
        let device_name: Option<String> = {
            let mut it = std::env::args();
            let mut out = None;
            while let Some(a) = it.next() {
                if a == "--device" { out = it.next(); }
            }
            out
        };
        let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt)?;
        eprintln!(">> (inline-leak) stream started");
        let join = std::thread::spawn(move || {
            let mut cons = handle.consumer;
            let mut buf = [0f32; 1024];
            let start = Instant::now();
            let mut last = Instant::now();
            while start.elapsed() < Duration::from_secs(5) {
                let n = cons.pop_slice(&mut buf);
                if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
                else { SAMPLES.fetch_add(n, Ordering::Relaxed); }
                if last.elapsed() >= Duration::from_secs(1) {
                    eprintln!(">> @{}s: samples={}",
                              start.elapsed().as_secs(),
                              SAMPLES.load(Ordering::Relaxed));
                    last = Instant::now();
                }
            }
            eprintln!(">> total: {}", SAMPLES.load(Ordering::Relaxed));
        });
        Box::leak(Box::new(join));
        if std::env::args().any(|a| a == "--drop-devname") {
            eprintln!(">> explicit drop(device_name)");
            drop(device_name);
        }
        eprintln!(">> main: sleeping 6s after Box::leak");
        std::thread::sleep(Duration::from_secs(6));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--inline-no-block") {
        // Inline equivalent of via_lib_flow_no_block + sleep, all in main's
        // stack frame.  This removes the function-return/unwind hypothesis.
        use ringbuf::traits::Consumer;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static SAMPLES: AtomicUsize = AtomicUsize::new(0);
        let device_name: Option<String> = {
            let mut it = std::env::args();
            let mut out = None;
            while let Some(a) = it.next() {
                if a == "--device" { out = it.next(); }
            }
            out
        };
        let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt)?;
        eprintln!(">> (inline) stream started; spawning pipeline thread");
        let _join = std::thread::spawn(move || {
            eprintln!(">> (pipeline-bg) thread entered; polling");
            let mut cons = handle.consumer;
            let mut buf = [0f32; 1024];
            let start = Instant::now();
            let mut last = Instant::now();
            while start.elapsed() < Duration::from_secs(5) {
                let n = cons.pop_slice(&mut buf);
                if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
                else { SAMPLES.fetch_add(n, Ordering::Relaxed); }
                if last.elapsed() >= Duration::from_secs(1) {
                    eprintln!(">> (pipeline-bg) @{}s: samples={}",
                              start.elapsed().as_secs(),
                              SAMPLES.load(Ordering::Relaxed));
                    last = Instant::now();
                }
            }
            eprintln!(">> (pipeline-bg) total samples drained: {}",
                      SAMPLES.load(Ordering::Relaxed));
        });
        eprintln!(">> main: sleeping 6s (holding _join in scope)");
        std::thread::sleep(Duration::from_secs(6));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--no-consumer") {
        // Create stream and return the handle back up; nobody drains the
        // ringbuf.  Just verify the callback fires at all.
        let device_name: Option<String> = std::env::args()
            .skip_while(|a| a != "--device")
            .nth(1);
        let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let _handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt)?;
        eprintln!(">> stream started, no consumer, sleeping 4s");
        std::thread::sleep(Duration::from_secs(4));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--keep-handle-in-main") {
        // Keep AudioHandle on main thread.  Main also drains the consumer
        // (moved from handle), so _stream stays in the partially-moved
        // handle alive on main.  This mirrors the WORKING daemon path as
        // closely as possible without extra threads.
        use ringbuf::traits::Consumer;
        let device_name: Option<String> = std::env::args()
            .skip_while(|a| a != "--device")
            .nth(1);
        let ptt = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt)?;
        let mut cons = handle.consumer;
        let mut buf = [0f32; 1024];
        let mut total = 0usize;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            let n = cons.pop_slice(&mut buf);
            if n == 0 { std::thread::sleep(Duration::from_millis(5)); }
            else { total += n; }
        }
        eprintln!(">> main drained {total} samples");
        return Ok(());
    }
    if std::env::args().any(|a| a == "--audio-single-thread-rt") {
        // Use current-thread runtime (no worker threads) instead of multi_thread.
        via_lib_flow_no_block()?;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        return rt.block_on(async {
            tokio::time::sleep(Duration::from_secs(6)).await;
            Ok(())
        });
    }
    if std::env::args().any(|a| a == "--via-lib-tokio-select") {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let no_sig = std::env::args().any(|a| a == "--no-sig");
        let sig_before_audio = std::env::args().any(|a| a == "--sig-before-audio");
        return rt.block_on(async {
            use tokio::signal::unix::{signal, SignalKind};
            if sig_before_audio {
                // Install signal handlers BEFORE starting audio stream, so
                // tokio's signal driver's sigaction is in place first.
                let _ = signal(SignalKind::terminate())?;
                let _ = signal(SignalKind::interrupt())?;
                eprintln!(">> signals registered BEFORE audio");
            }
            via_lib_flow_no_block()?;
            if no_sig {
                eprintln!(">> skipping signal setup; select with timeout only");
                tokio::time::sleep(Duration::from_secs(6)).await;
                return Ok(());
            }
            let mut sigterm = signal(SignalKind::terminate())?;
            let mut sigint = signal(SignalKind::interrupt())?;
            eprintln!(">> entering tokio::select! for 5s");
            let deadline = tokio::time::sleep(Duration::from_secs(6));
            tokio::select! {
                _ = sigterm.recv() => eprintln!(">> SIGTERM"),
                _ = sigint.recv()  => eprintln!(">> SIGINT"),
                _ = deadline       => eprintln!(">> 5s deadline"),
            }
            Ok(())
        });
    }
    if std::env::args().any(|a| a == "--via-lib") {
        return via_lib_flow();
    }
    if std::env::args().any(|a| a == "--__unused") {
        let device_name: Option<String> = {
            let mut it = std::env::args();
            let mut out = None;
            while let Some(a) = it.next() {
                if a == "--device" {
                    out = it.next();
                }
            }
            out
        };
        let do_uinput = std::env::args().any(|a| a == "--do-uinput");
        let do_evdev = std::env::args().any(|a| a == "--do-evdev");
        let do_inject = std::env::args().any(|a| a == "--do-inject");

        if do_uinput {
            eprintln!(">> opening uinput device...");
            let _kbd = vibe_attack::input::inject::open_uinput_device()?;
            eprintln!(">> uinput opened (held alive for the rest of this run)");
            if do_inject {
                use std::sync::mpsc;
                eprintln!(">> spawning injection thread...");
                let (_tx, rx) = mpsc::channel::<vibe_attack::input::inject::MacroCmd>();
                let _h = vibe_attack::input::inject::spawn_injection_thread(_kbd, rx);
                eprintln!(">> injection thread spawned");
            } else {
                std::mem::forget(_kbd); // keep alive
            }
        }
        if do_evdev {
            eprintln!(">> scanning evdev for PTT device...");
            let key = vibe_attack::input::ptt::parse_key_code("KEY_LEFTCTRL")?;
            vibe_attack::input::ptt::check_input_readable()?;
            let _dev = vibe_attack::input::ptt::find_ptt_device(key)?;
            eprintln!(">> evdev PTT device found");
            std::mem::forget(_dev);
        }

        let do_pipeline = std::env::args().any(|a| a == "--do-pipeline-thread");
        let ptt_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt_active)?;
        eprintln!(">> lib-path stream started");

        if do_pipeline {
            // Mirror daemon architecture: move AudioHandle into pipeline
            // thread which loads Silero VAD then polls consumer.
            let join = std::thread::spawn(move || {
                eprintln!(">> (pipeline) thread started; loading Silero VAD...");
                let _vad = silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
                    silero_vad_rust::silero_vad::model::LoadOptions {
                        force_onnx_cpu: true,
                        ..Default::default()
                    },
                )
                .expect("load silero");
                eprintln!(">> (pipeline) Silero VAD loaded; polling ringbuf");
                let mut cons = handle.consumer;
                let mut buf = [0f32; 1024];
                use ringbuf::traits::Consumer;
                let mut total = 0;
                let start = Instant::now();
                while start.elapsed() < Duration::from_secs(4) {
                    let n = cons.pop_slice(&mut buf);
                    if n == 0 {
                        std::thread::sleep(Duration::from_millis(5));
                    } else {
                        total += n;
                    }
                }
                eprintln!(">> (pipeline) samples drained from ringbuf in 4s: {total}");
            });
            join.join().ok();
            return Ok(());
        }

        eprintln!(">> sleeping 4s");
        std::thread::sleep(Duration::from_secs(4));
        let mut cons = handle.consumer;
        let mut buf = [0f32; 1024];
        use ringbuf::traits::Consumer;
        let mut total = 0;
        for _ in 0..50 {
            total += cons.pop_slice(&mut buf);
        }
        eprintln!(">> samples drained from ringbuf after 4s: {total}");
        return Ok(());
    }
    if std::env::args().any(|a| a == "--full-daemon") {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        return rt.block_on(async {
            use tokio::signal::unix::{signal, SignalKind};
            let _sig_term = signal(SignalKind::terminate())?;
            let _sig_int = signal(SignalKind::interrupt())?;
            eprintln!(">> tokio signal handlers installed");
            real_main()
        });
    }
    if std::env::args().any(|a| a == "--with-tokio") {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async { real_main() });
    }
    real_main()
}

// Same as via_lib_flow but starts the pipeline thread in the background and
// returns immediately (for emulating the daemon main-thread blocking on
// signals).  Uses a global atomic to drain samples count.
fn via_lib_flow_no_block() -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SAMPLES: AtomicUsize = AtomicUsize::new(0);
    let device_name: Option<String> = {
        let mut it = std::env::args();
        let mut out = None;
        while let Some(a) = it.next() {
            if a == "--device" {
                out = it.next();
            }
        }
        out
    };
    let ptt_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handle = vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt_active)?;
    eprintln!(">> (no-block) stream started; spawning pipeline thread");
    let load_vad_in_thread = std::env::args().any(|a| a == "--bg-load-vad");
    let join = std::thread::spawn(move || {
        use ringbuf::traits::Consumer;
        eprintln!(">> (pipeline-bg) thread entered");
        if load_vad_in_thread {
            eprintln!(">> (pipeline-bg) loading Silero VAD...");
            let _vad = silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
                silero_vad_rust::silero_vad::model::LoadOptions {
                    force_onnx_cpu: true,
                    ..Default::default()
                },
            )
            .expect("load silero");
            eprintln!(">> (pipeline-bg) Silero VAD loaded");
        }
        let mut cons = handle.consumer;
        let mut buf = [0f32; 1024];
        let start = Instant::now();
        let mut last_heartbeat = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            let n = cons.pop_slice(&mut buf);
            if n == 0 {
                std::thread::sleep(Duration::from_millis(5));
            } else {
                SAMPLES.fetch_add(n, Ordering::Relaxed);
            }
            if last_heartbeat.elapsed() >= Duration::from_secs(1) {
                eprintln!(
                    ">> (pipeline-bg) @{}s: samples={}",
                    start.elapsed().as_secs(),
                    SAMPLES.load(Ordering::Relaxed)
                );
                last_heartbeat = Instant::now();
            }
        }
        eprintln!(
            ">> (pipeline-bg) total samples drained: {}",
            SAMPLES.load(Ordering::Relaxed)
        );
    });
    if std::env::args().any(|a| a == "--sleep-inside") {
        eprintln!(">> (no-block) sleeping INSIDE helper for 6s");
        std::thread::sleep(Duration::from_secs(6));
    }
    if std::env::args().any(|a| a == "--sleep-100ms-inside") {
        eprintln!(">> (no-block) sleeping 100ms INSIDE helper before returning");
        std::thread::sleep(Duration::from_millis(100));
    }
    Box::leak(Box::new(join));
    Ok(())
}

fn via_lib_flow() -> anyhow::Result<()> {
    let device_name: Option<String> = {
        let mut it = std::env::args();
        let mut out = None;
        while let Some(a) = it.next() {
            if a == "--device" {
                out = it.next();
            }
        }
        out
    };
    let do_uinput = std::env::args().any(|a| a == "--do-uinput");
    let do_evdev = std::env::args().any(|a| a == "--do-evdev");
    let do_inject = std::env::args().any(|a| a == "--do-inject");

    if do_uinput {
        eprintln!(">> opening uinput device...");
        let _kbd = vibe_attack::input::inject::open_uinput_device()?;
        eprintln!(">> uinput opened");
        if do_inject {
            use std::sync::mpsc;
            eprintln!(">> spawning injection thread...");
            let (_tx, rx) = mpsc::channel::<vibe_attack::input::inject::MacroCmd>();
            let _h = vibe_attack::input::inject::spawn_injection_thread(_kbd, rx);
            eprintln!(">> injection thread spawned");
        } else {
            std::mem::forget(_kbd);
        }
    }
    if do_evdev {
        eprintln!(">> scanning evdev for PTT device...");
        let key = vibe_attack::input::ptt::parse_key_code("KEY_LEFTCTRL")?;
        vibe_attack::input::ptt::check_input_readable()?;
        let _dev = vibe_attack::input::ptt::find_ptt_device(key)?;
        eprintln!(">> evdev PTT device found");
        std::mem::forget(_dev);
    }

    let do_pipeline = std::env::args().any(|a| a == "--do-pipeline-thread");
    let ptt_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let handle =
        vibe_attack::audio::start_audio_stream(device_name.as_deref(), ptt_active)?;
    eprintln!(">> lib-path stream started");

    if do_pipeline {
        let join = std::thread::spawn(move || {
            eprintln!(">> (pipeline) thread started; loading Silero VAD...");
            let _vad = silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
                silero_vad_rust::silero_vad::model::LoadOptions {
                    force_onnx_cpu: true,
                    ..Default::default()
                },
            )
            .expect("load silero");
            eprintln!(">> (pipeline) Silero VAD loaded; polling ringbuf");
            let mut cons = handle.consumer;
            let mut buf = [0f32; 1024];
            use ringbuf::traits::Consumer;
            let mut total = 0;
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(4) {
                let n = cons.pop_slice(&mut buf);
                if n == 0 {
                    std::thread::sleep(Duration::from_millis(5));
                } else {
                    total += n;
                }
            }
            eprintln!(">> (pipeline) samples drained in 4s: {total}");
        });
        if std::env::args().any(|a| a == "--main-sleep-not-join") {
            eprintln!(">> main: sleeping 5s (NOT joining) while pipeline runs");
            std::thread::sleep(Duration::from_secs(5));
            eprintln!(">> main: woke up; pipeline thread may still be running");
        } else {
            join.join().ok();
        }
        return Ok(());
    }

    eprintln!(">> sleeping 4s");
    std::thread::sleep(Duration::from_secs(4));
    let mut cons = handle.consumer;
    let mut buf = [0f32; 1024];
    use ringbuf::traits::Consumer;
    let mut total = 0;
    for _ in 0..50 {
        total += cons.pop_slice(&mut buf);
    }
    eprintln!(">> samples drained after 4s: {total}");
    Ok(())
}

fn real_main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut device_name: Option<String> = None;
    let mut secs: u64 = 5;
    let mut list = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" | "-l" => list = true,
            "--device" | "-d" => {
                i += 1;
                device_name = args.get(i).cloned();
            }
            "--secs" | "-s" => {
                i += 1;
                secs = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(5);
            }
            "--with-ort" => {}
            "--move-to-thread" => {}
            "--with-tokio" => {}
            "--full-daemon" => {}
            "--via-lib" => {}
            "--do-uinput" => {}
            "--do-evdev" => {}
            "--do-inject" => {}
            "--post-ort-in-thread" => {}
            "--do-pipeline-thread" => {}
            "--via-lib-tokio-select" => {}
            "--no-sig" => {}
            "--sig-before-audio" => {}
            "--audio-before-runtime" => {}
            "--audio-single-thread-rt" => {}
            "--no-tokio-wait" => {}
            "--no-consumer" => {}
            "--keep-handle-in-main" => {}
            "--bg-load-vad" => {}
            "--main-sleep-not-join" => {}
            "--inline-no-block" => {}
            "--inline-nested-block" => {}
            "--keep-ptt" => {}
            "--inline-leak" => {}
            "--inline-leak-block" => {}
            "--drop-devname" => {}
            "--inline-leak-block-minimal" => {}
            "--audio-in-block-spawn-out" => {}
            "--sleep-inside" => {}
            "--sleep-100ms-inside" => {}
            other => {
                eprintln!("Unknown arg: {other}");
                print_usage();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    // Optional: load ONNX Runtime (Silero VAD) BEFORE opening the stream.  This
    // mirrors what the real daemon does and lets us test whether ort init
    // breaks the CPAL audio thread.  Enable with --with-ort on the CLI.
    let with_ort = std::env::args().any(|a| a == "--with-ort");
    if with_ort {
        eprintln!(">> pre-loading Silero VAD (ort)...");
        let _ = silero_vad_rust::silero_vad::model::load_silero_vad_with_options(
            silero_vad_rust::silero_vad::model::LoadOptions {
                force_onnx_cpu: true,
                ..Default::default()
            },
        )
        .expect("load silero VAD");
        eprintln!(">> Silero VAD loaded");
    }

    let host = cpal::default_host();

    if list {
        println!("Available input devices:");
        for d in host.input_devices()? {
            let name = d
                .description()
                .map(|x| x.name().to_string())
                .unwrap_or_else(|_| "<?>".into());
            let cfg = d
                .default_input_config()
                .map(|c| format!("{} ch @ {} Hz", c.channels(), c.sample_rate()))
                .unwrap_or_else(|_| "<?>".into());
            println!("  {name}  ({cfg})");
        }
        return Ok(());
    }

    let device = match device_name.as_deref() {
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No default input device"))?,
        Some(target) => host
            .input_devices()?
            .find(|d| {
                d.description()
                    .map(|x| x.name() == target)
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow::anyhow!("Device not found: {target}"))?,
    };

    let name = device
        .description()
        .map(|x| x.name().to_string())
        .unwrap_or_else(|_| "<?>".into());
    let default_cfg = device.default_input_config()?;
    println!(
        "Using device: {name}  (native {} ch @ {} Hz, {:?})",
        default_cfg.channels(),
        default_cfg.sample_rate(),
        default_cfg.sample_format()
    );

    let mut config: cpal::StreamConfig = default_cfg.into();
    config.buffer_size = cpal::BufferSize::Fixed(1024);
    println!(
        "Opening stream: {} ch @ {} Hz, buffer Fixed(1024)",
        config.channels, config.sample_rate
    );

    let sample_count = Arc::new(AtomicUsize::new(0));
    let callback_count = Arc::new(AtomicUsize::new(0));
    let sc = Arc::clone(&sample_count);
    let cc = Arc::clone(&callback_count);
    let first = Arc::new(AtomicUsize::new(0));
    let first_cb = Arc::clone(&first);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _info: &cpal::InputCallbackInfo| {
            if first_cb.fetch_add(1, Ordering::Relaxed) == 0 {
                eprintln!(">> first callback: {} samples", data.len());
            }
            cc.fetch_add(1, Ordering::Relaxed);
            sc.fetch_add(data.len(), Ordering::Relaxed);
        },
        |err| eprintln!("!! CPAL stream error: {err}"),
        None,
    )?;

    stream.play()?;

    // If --move-to-thread is given, move the Stream into a spawned OS thread
    // (mirroring the daemon's spawn_pipeline architecture) and let that thread
    // hold the stream alive.  Tests whether moving cpal::Stream across threads
    // breaks the audio callback.
    let move_to_thread = std::env::args().any(|a| a == "--move-to-thread");
    let secs_copy = secs;
    let sc2 = Arc::clone(&sample_count);
    let cc2 = Arc::clone(&callback_count);
    let join = if move_to_thread {
        eprintln!(">> moving stream into spawned thread");
        Some(std::thread::spawn(move || {
            // Hold the stream in this thread's stack until sleep finishes.
            let _keep = stream;
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(secs_copy) {
                std::thread::sleep(Duration::from_millis(250));
                let n = sc2.load(Ordering::Relaxed);
                let c = cc2.load(Ordering::Relaxed);
                eprint!("\r  callbacks={c:4}  samples={n:8}          ");
            }
            eprintln!();
            drop(_keep);
        }))
    } else {
        println!("Capturing for {secs}s...");
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(secs) {
            std::thread::sleep(Duration::from_millis(250));
            let n = sample_count.load(Ordering::Relaxed);
            let c = callback_count.load(Ordering::Relaxed);
            eprint!("\r  callbacks={c:4}  samples={n:8}          ");
        }
        eprintln!();
        drop(stream);
        None
    };
    if let Some(j) = join {
        j.join().ok();
    }

    let total = sample_count.load(Ordering::Relaxed);
    let cbs = callback_count.load(Ordering::Relaxed);
    println!("\nResult:");
    println!("  callbacks fired : {cbs}");
    println!("  samples total   : {total}");
    let expected = (config.sample_rate as u64 * secs * config.channels as u64) as usize;
    println!("  samples expected: ~{expected}");
    if total == 0 {
        println!("  VERDICT: FAILED — CPAL opened stream but host delivered zero samples.");
    } else if (total as f64) < (expected as f64 * 0.5) {
        println!("  VERDICT: DEGRADED — stream stalled or delivered far fewer samples than expected.");
    } else {
        println!("  VERDICT: OK — audio capture is working on this device.");
    }
    Ok(())
}

fn print_usage() {
    eprintln!("Usage: audio_probe [--list] [--device NAME] [--secs N]");
}
