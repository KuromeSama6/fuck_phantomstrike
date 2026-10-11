use std::cell::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command()]
pub struct Config {
    /// The path to the Steam installation directory.
    pub steam_path: PathBuf,

    /// The frequency (in milliseconds) at which to read from the log file.
    #[arg(short, long, default_value_t = 50)]
    pub update_frequency_ms: u64,

    #[arg(long, default_value_t = 100)]
    pub parry_delay_min_ms: u64,
    #[arg(long, default_value_t = 200)]
    pub parry_delay_max_ms: u64,
    
    #[arg(long, default_value_t = 0x21)]
    pub parry_key_code: u16,
}

impl Config {
    pub fn get_console_path(&self) -> PathBuf {
        self.steam_path.join("steamapps/common/Deadlock/game/citadel/console.log")
    }
}

static CONFIG: OnceLock<Arc<Config>> = OnceLock::new();

pub fn get() -> Arc<Config> {
    if let Some(config) = CONFIG.get() {
        return config.clone();
    }

    let config = Arc::new(Config::parse());
    CONFIG.set(config.clone()).unwrap();
    config
}
