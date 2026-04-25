use clap::Parser;
use std::io::IsTerminal;
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

    /// List available audio input devices and exit (use the name in audio.device in config.yaml)
    #[arg(long)]
    list_devices: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Check if the daemon is alive
    Ping,
    /// Switch the active macro pack/profile
    Switch { name: String },
    /// Execute a specific macro by name immediately (for testing)
    Test { name: String },
    /// Import a .hdpack file
    Import { file: std::path::PathBuf },
    /// Export the current profile to a .hdpack file
    Export { name: String, output: Option<std::path::PathBuf> },
    /// Open the interactive TUI editor
    Edit,
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

    // IMPORTANT: stdout is reserved for machine-readable JSONL.
    // Always write logs to stderr, and only use ANSI colors when stderr is a TTY.
    let ansi = std::io::stderr().is_terminal();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(ansi)
        .with_writer(std::io::stderr)
        .compact()
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::{atomic::AtomicBool, mpsc, Arc};
    use tokio_util::sync::CancellationToken;

    let args = Cli::parse();
    init_logging(args.verbose);

    // --list-devices: enumerate CPAL input devices and exit.
    if args.list_devices {
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();
        println!("Available audio input devices (use name in audio.device in config.yaml):");
        println!();
        match host.input_devices() {
            Ok(devices) => {
                let mut found = false;
                for (idx, device) in devices.enumerate() {
                    found = true;
                    let cfg = device.default_input_config()
                        .map(|c| format!("{} ch @ {} Hz", c.channels(), c.sample_rate().0))
                        .unwrap_or_else(|_| "no supported config".to_string());
                    println!("  Device {}: {}", idx, cfg);
                }
                if !found {
                    println!("  (no input devices found)");
                }
            }
            Err(e) => eprintln!("Failed to enumerate devices: {e}"),
        }
        return Ok(());
    }

    if let Some(cmd) = args.command {
        use hd_linux_voice::control::protocol::ControlRequest;
        use hd_linux_voice::control::client::send_command;

        match cmd {
            Commands::Ping => {
                match send_command(ControlRequest::Ping) {
                    Ok(resp) => println!("{resp:?}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Commands::Switch { name } => {
                match send_command(ControlRequest::SwitchProfile { name }) {
                    Ok(resp) => println!("{resp:?}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Commands::Test { name } => {
                match send_command(ControlRequest::TestMacro { name }) {
                    Ok(resp) => println!("{resp:?}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Commands::Import { file } => {
                use hd_linux_voice::pack::Pack;
                match Pack::import(&file) {
                    Ok(pack) => println!("Successfully imported pack: {}", pack.name),
                    Err(e) => eprintln!("Import error: {e:#}"),
                }
            }
            Commands::Export { name, output } => {
                use hd_linux_voice::pack::{Pack, get_profiles_dir};
                let dir = get_profiles_dir().unwrap().join(&name);
                match Pack::load_from_dir(&dir) {
                    Ok(pack) => {
                        let out = output.unwrap_or_else(|| std::path::PathBuf::from(format!("{name}.hdpack")));
                        match pack.export(&dir, &out) {
                            Ok(_) => println!("Successfully exported pack to: {}", out.display()),
                            Err(e) => eprintln!("Export error: {e:#}"),
                        }
                    }
                    Err(e) => eprintln!("Error loading profile '{name}': {e:#}"),
                }
            }
            Commands::Edit => {
                hd_linux_voice::tui::run_editor().unwrap();
            }
        }
        return Ok(());
    }

    tracing::info!("hd-linux-voice starting");

    // === 1. Load config (fail-hard on any error) ===
    let mut config = hd_linux_voice::config::load(args.config.as_deref()).map_err(|e| {
        eprintln!("{e:#}");
        e
    })?;

    // === 1a. Integrate ProfileManager (if no explicit config passed) ===
    if args.config.is_none() {
        use hd_linux_voice::pack::manager::ProfileManager;
        let manager = ProfileManager::load()?;
        if let Some(pack) = manager.get_active_pack()? {
            tracing::info!(profile = %pack.name, macros = pack.flatten().len(), "Applying active profile macros");
            config.macros = pack.flatten();
        }
    }

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
    let audio_handle = hd_linux_voice::audio::start_audio_stream(
        config.audio.device.as_deref(),
        Arc::clone(&ptt_active),
    )
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
    //
    // Split the audio handle: the CPAL stream guard MUST remain on this
    // (main) thread.  Moving `cpal::Stream` into a worker thread that is
    // spawned from inside a nested helper and then dropped has been shown
    // to silently stop the ALSA/PipeWire callback on Linux.  Pass only the
    // ringbuf consumer into the pipeline.
    let hd_linux_voice::audio::AudioHandle {
        stream: _audio_stream_guard,
        consumer: audio_consumer,
        actual_config: _actual_cfg,
    } = audio_handle;

    let pipeline_handles = hd_linux_voice::pipeline::coordinator::spawn_pipeline(
        audio_consumer,
        config.clone(),
        Arc::clone(&ptt_active),
        macro_tx.clone(),
        shutdown.clone(),
    )
    .map_err(|e| {
        // If pipeline preflight fails, stop injection thread before exiting.
        let _ = macro_tx.send(hd_linux_voice::input::inject::MacroCmd::Shutdown);
        eprintln!("{e:#}");
        e
    })?;

    // === 10b. Spawn UDS control listener (Phase 4) ===
    hd_linux_voice::control::spawn_control_listener(pipeline_handles.dispatcher.clone()).await.map_err(|e| {
        tracing::error!("Failed to start control listener: {e:#}");
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

    // Dropping _audio_stream_guard here stops the CPAL stream (RAII).
    drop(_audio_stream_guard);

    tracing::info!("hd-linux-voice stopped");
    Ok(())
}
