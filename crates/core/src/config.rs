//! Nexus configuration loaded from TOML.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub watcher: WatcherConfig,
    #[serde(default)]
    pub ai: AiConfig,
}

/// Scan configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Root directory to scan (default: ~).
    pub root: Option<PathBuf>,
    /// Directories to exclude from scanning.
    #[serde(default = "default_excludes")]
    pub excludes: Vec<String>,
    /// Maximum depth to scan (0 = unlimited).
    #[serde(default)]
    pub max_depth: usize,
    /// Follow symlinks.
    #[serde(default)]
    pub follow_symlinks: bool,
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database.
    pub path: Option<PathBuf>,
    /// Cache size in MB.
    #[serde(default = "default_cache_mb")]
    pub cache_mb: u32,
    /// Mmap size in MB.
    #[serde(default = "default_mmap_mb")]
    pub mmap_mb: u32,
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

/// Watcher / daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Directories to watch (default: ~ and ~/.config).
    #[serde(default)]
    pub watch_paths: Vec<PathBuf>,
    /// Auto-snapshot configs on change.
    #[serde(default = "default_true")]
    pub auto_snapshot: bool,
    /// Debounce interval in seconds.
    #[serde(default = "default_debounce")]
    pub debounce_secs: u64,
}

/// AI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Claude API key (or read from ANTHROPIC_API_KEY env).
    pub api_key: Option<String>,
    /// Model to use.
    #[serde(default = "default_model")]
    pub model: String,
    /// Max tokens for responses.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            root: None,
            excludes: default_excludes(),
            max_depth: 0,
            follow_symlinks: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: None,
            cache_mb: default_cache_mb(),
            mmap_mb: default_mmap_mb(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            watch_paths: Vec::new(),
            auto_snapshot: true,
            debounce_secs: default_debounce(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: default_model(),
            max_tokens: default_max_tokens(),
        }
    }
}

fn default_excludes() -> Vec<String> {
    vec![
        "Library".into(),
        ".Trash".into(),
        "node_modules".into(),
        "target".into(),
        ".git".into(),
        "__pycache__".into(),
        ".cache".into(),
        ".npm".into(),
        ".cargo/registry".into(),
    ]
}

fn default_cache_mb() -> u32 {
    64
}
fn default_mmap_mb() -> u32 {
    256
}
fn default_port() -> u16 {
    3141
}
fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_debounce() -> u64 {
    5
}
fn default_model() -> String {
    "claude-sonnet-4-20250514".into()
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_true() -> bool {
    true
}

/// Default config directory: `~/.config/nexus/`.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
}

/// Default config file path: `~/.config/nexus/config.toml`.
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Load configuration from the default path, falling back to defaults.
pub fn load() -> Result<Config> {
    load_from(&config_path())
}

/// Load configuration from a specific path.
pub fn load_from(path: &Path) -> Result<Config> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::error::NexusError::Config(e.to_string()))?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

/// Write default config to the config path if it doesn't exist.
pub fn init() -> Result<PathBuf> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let config = Config::default();
        let content = toml::to_string_pretty(&config)
            .map_err(|e| crate::error::NexusError::Config(e.to_string()))?;
        std::fs::write(&path, content)?;
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let parsed: Config = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(parsed.server.port, 3141);
        assert_eq!(parsed.database.cache_mb, 64);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let config = load_from(Path::new("/nonexistent/config.toml")).unwrap();
        assert_eq!(config.server.port, 3141);
    }
}
