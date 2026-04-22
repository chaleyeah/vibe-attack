use anyhow::Result;
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

fn main() -> Result<()> {
    let args = Cli::parse();
    init_logging(args.verbose);

    tracing::debug!("hd-linux-voice starting");

    // Load config — fail fast on any error (D-11, D-15 policy applies to all startup failures)
    let config_path = args.config.as_deref();
    let _config = hd_linux_voice::config::load(config_path).map_err(|e| {
        // Print the full error chain for actionable diagnostics
        eprintln!("Error loading config: {e:#}");
        e
    })?;

    tracing::info!("Config loaded successfully");

    // Daemon loop: spawned in Plan 05 (01-05-PLAN.md)
    tracing::info!("Daemon stub: exiting (full daemon loop added in Plan 05)");
    Ok(())
}
