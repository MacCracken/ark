//! TOML configuration file loading for ark.
//!
//! Search order: `$ARK_CONFIG` env var, `./ark.toml`, `~/.config/ark/ark.toml`,
//! `/etc/agnos/ark.toml`, then built-in defaults.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{debug, info};

use crate::types::ArkConfig;

/// Partial config for TOML deserialization — all fields optional.
/// Missing fields fall back to `ArkConfig::default()`.
#[derive(Debug, Deserialize, Default)]
struct PartialArkConfig {
    default_strategy: Option<nous::ResolutionStrategy>,
    confirm_system_installs: Option<bool>,
    confirm_removals: Option<bool>,
    auto_update_check: Option<bool>,
    color_output: Option<bool>,
    marketplace_dir: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    package_db_path: Option<PathBuf>,
    transaction_log_path: Option<PathBuf>,
}

impl PartialArkConfig {
    fn merge_into(self, base: &mut ArkConfig) {
        if let Some(v) = self.default_strategy {
            base.default_strategy = v;
        }
        if let Some(v) = self.confirm_system_installs {
            base.confirm_system_installs = v;
        }
        if let Some(v) = self.confirm_removals {
            base.confirm_removals = v;
        }
        if let Some(v) = self.auto_update_check {
            base.auto_update_check = v;
        }
        if let Some(v) = self.color_output {
            base.color_output = v;
        }
        if let Some(v) = self.marketplace_dir {
            base.marketplace_dir = v;
        }
        if let Some(v) = self.cache_dir {
            base.cache_dir = v;
        }
        if let Some(v) = self.package_db_path {
            base.package_db_path = v;
        }
        if let Some(v) = self.transaction_log_path {
            base.transaction_log_path = v;
        }
    }
}

/// Load config from a specific TOML file, merging with defaults.
pub fn load_config_from(path: &Path) -> Result<ArkConfig> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file {}", path.display()))?;
    let partial: PartialArkConfig = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file {}", path.display()))?;
    let mut config = ArkConfig::default();
    partial.merge_into(&mut config);
    info!(path = %path.display(), "Loaded configuration");
    Ok(config)
}

/// Load config using the standard search order.
///
/// 1. `$ARK_CONFIG` environment variable
/// 2. `./ark.toml`
/// 3. `~/.config/ark/ark.toml`
/// 4. `/etc/agnos/ark.toml`
/// 5. Built-in defaults
pub fn load_config() -> Result<ArkConfig> {
    // 1. Env var
    if let Ok(path) = std::env::var("ARK_CONFIG") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return load_config_from(&p);
        }
        debug!(path = %path, "ARK_CONFIG set but file not found, continuing search");
    }

    // 2-4. Search paths
    let mut search_paths = vec![PathBuf::from("./ark.toml")];
    if let Some(home) = std::env::var_os("HOME") {
        search_paths.push(PathBuf::from(home).join(".config/ark/ark.toml"));
    }
    search_paths.push(PathBuf::from("/etc/agnos/ark.toml"));

    for path in &search_paths {
        if path.exists() {
            return load_config_from(path);
        }
    }

    // 5. Defaults
    debug!("No config file found, using defaults");
    Ok(ArkConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_config_from_toml_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp.as_file(),
            r#"
            color_output = false
            confirm_removals = false
            marketplace_dir = "/opt/marketplace"
            "#
        )
        .unwrap();

        let config = load_config_from(tmp.path()).unwrap();
        assert!(!config.color_output);
        assert!(!config.confirm_removals);
        assert_eq!(config.marketplace_dir, PathBuf::from("/opt/marketplace"));
        // Defaults should be preserved for unset fields
        assert!(config.confirm_system_installs);
    }

    #[test]
    fn load_config_missing_file_returns_defaults() {
        let config = load_config().unwrap();
        assert!(config.color_output);
        assert!(config.confirm_system_installs);
    }

    #[test]
    fn load_config_empty_toml_returns_defaults() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp.as_file()).unwrap();
        let config = load_config_from(tmp.path()).unwrap();
        assert_eq!(config, ArkConfig::default());
    }
}
