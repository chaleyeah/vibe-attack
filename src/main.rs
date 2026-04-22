use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
    name = "hd-linux-voice",
    about = "Voice-macro daemon for Helldivers 2 on Linux",
    long_about = None,
    version,
)]
struct Cli {
    /// Enable verbose (DEBUG) logging. Repeat for TRACE: -vv
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Path to config file (default: $XDG_CONFIG_HOME/hd-linux-voice/config.yaml)
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
}

fn init_logging(verbose: u8) {
    // D-08 / D-12: silent by default; DEBUG at -v; TRACE at -vv
    let level = match verbose {
        0 => "warn",
        1 => "debug",
        _ => "trace",
    };
    // Allow RUST_LOG env var to override (standard Rust logging convention)
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::{atomic::AtomicBool, mpsc, Arc};
    use tokio_util::sync::CancellationToken;

    let args = Cli::parse();
    init_logging(args.verbose);

    tracing::info!("hd-linux-voice starting");

    // === 1. Load config (fail-hard on any error) ===
    let config = hd_linux_voice::config::load(args.config.as_deref()).map_err(|e| {
        eprintln!("{e:#}");
        e
    })?;

    tracing::info!(
        ptt_key = %config.ptt.key,
        dwell_ms = config.timing.dwell_ms,
        gap_ms = config.timing.gap_ms,
        macros = config.macros.len(),
        "Config loaded"
    );

    // === 1b. Preflight: validate all local model paths BEFORE threads (Phase 2) ===
    config.validate_model_paths().map_err(|e| {
        eprintln!("{e:#}");
        e
    })?;

    // === 2. Parse PTT key code ===
    let ptt_key = hd_linux_voice::input::ptt::parse_key_code(&config.ptt.key)
        .map_err(|e| { eprintln!("{e:#}"); e })?;

    // === 3. Preflight: verify /dev/input readable (fail-hard, D-11) ===
    hd_linux_voice::input::ptt::check_input_readable().map_err(|e| {
        eprintln!("{e:#}");
        e
    })?;

    // === 4. Open uinput virtual keyboard (fail-hard, D-15) ===
    // Must happen BEFORE spawning any threads — if it fails, we exit cleanly.
    let virtual_kbd = hd_linux_voice::input::inject::open_uinput_device().map_err(|e| {
        eprintln!("{e:#}");
        e
    })?;
    tracing::info!("uinput virtual keyboard opened");

    // === 5. Find PTT device ===
    let ptt_device = hd_linux_voice::input::ptt::find_ptt_device(ptt_key).map_err(|e| {
        eprintln!("{e:#}");
        e
    })?;

    // === 6. Shared state ===
    let ptt_active = Arc::new(AtomicBool::new(false));
    let shutdown = CancellationToken::new();

    // === 7. Spawn injection thread (std::thread, D-07) ===
    let (macro_tx, macro_rx) = mpsc::channel::<hd_linux_voice::input::inject::MacroCmd>();
    let inject_handle = hd_linux_voice::input::inject::spawn_injection_thread(virtual_kbd, macro_rx);

    // === 8. Start CPAL audio stream (warm, D-04) ===
    let audio_handle = hd_linux_voice::audio::start_audio_stream(Arc::clone(&ptt_active))
        .map_err(|e| {
            let _ = macro_tx.send(hd_linux_voice::input::inject::MacroCmd::Shutdown);
            eprintln!("Audio error: {e:#}");
            e
        })?;

    // === 9. Spawn PTT thread (std::thread, D-09, D-10) ===
    let ptt_handle = hd_linux_voice::input::ptt::spawn_ptt_thread(
        ptt_device,
        ptt_key,
        Arc::clone(&ptt_active),
        shutdown.clone(),
    );

    // === 10. Spawn Phase 2 pipeline (OS threads; stdout JSONL only) ===
    let pipeline_handles = hd_linux_voice::pipeline::coordinator::spawn_pipeline(
        audio_handle,
        config.clone(),
        Arc::clone(&ptt_active),
        shutdown.clone(),
    )
    .map_err(|e| {
        // If pipeline preflight fails, stop injection thread before exiting.
        let _ = macro_tx.send(hd_linux_voice::input::inject::MacroCmd::Shutdown);
        eprintln!("{e:#}");
        e
    })?;

    tracing::info!("Daemon running. Hold PTT and speak to emit JSONL transcripts. Ctrl+C or SIGTERM to exit.");

    // === 11. Wait for SIGTERM or SIGINT ===
    let mut sigterm = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate()
    )?;
    let mut sigint = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::interrupt()
    )?;

    tokio::select! {
        _ = sigterm.recv() => tracing::info!("SIGTERM received"),
        _ = sigint.recv()  => tracing::info!("SIGINT received (Ctrl+C)"),
    }

    // === 12. Graceful shutdown ===
    tracing::info!("Shutting down...");

    // Cancel PTT thread (checks is_cancelled() between event batches)
    shutdown.cancel();

    // Best-effort request STT shutdown + join pipeline threads (T-02-09).
    let mut pipeline_handles = pipeline_handles;

    if let Some(mut stt) = pipeline_handles.stt.take() {
        stt.request_shutdown();
        stt.join_best_effort(std::time::Duration::from_millis(500));
    }

    let pipeline_join = std::thread::spawn(move || pipeline_handles.pipeline.join());
    let output_join = std::thread::spawn(move || pipeline_handles.output.join());
    std::thread::sleep(std::time::Duration::from_millis(500));
    drop(pipeline_join);
    drop(output_join);

    // Signal injection thread and wait for it to drain
    let _ = macro_tx.send(hd_linux_voice::input::inject::MacroCmd::Shutdown);
    if let Err(e) = inject_handle.join() {
        tracing::warn!("Injection thread panicked: {e:?}");
    }

    // PTT thread: best-effort join — fetch_events() may block until next event
    let ptt_join = std::thread::spawn(move || ptt_handle.join());
    std::thread::sleep(std::time::Duration::from_millis(500));
    drop(ptt_join);

    // audio_handle.consumer was moved into the pipeline thread; dropping the handle stops CPAL stream.

    tracing::info!("hd-linux-voice stopped");
    Ok(())
}
