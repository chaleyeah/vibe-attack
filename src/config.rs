//! Configuration types — implemented in Phase 01 Plan 02.

pub struct Config;

pub fn load_config(_path: Option<&str>) -> anyhow::Result<Config> {
    Ok(Config)
}
