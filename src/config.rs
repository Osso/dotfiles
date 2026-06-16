use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::utils::expand_path;

/// TOML config for symlinks (config.toml)
#[derive(Deserialize)]
pub struct LinksConfig {
    pub source_dir: String,
    #[serde(default)]
    pub links: BTreeMap<String, String>,
    #[serde(default)]
    pub patterns: BTreeMap<String, String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl LinksConfig {
    pub fn load(path: &str) -> Result<Self> {
        let path = expand_path(path)?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        toml::from_str(&content).context("Failed to parse config")
    }
}

/// YAML config for system setup (setup.yaml)
#[derive(Deserialize, Default)]
pub struct SetupConfig {
    #[serde(default)]
    pub services: Services,
    #[serde(default)]
    pub directories: Vec<String>,
    #[serde(default)]
    pub modules: ModulesConfig,
    #[serde(default)]
    pub users: Vec<UserSpec>,
}

/// A declaratively-managed user account.
#[derive(Deserialize, Default)]
pub struct UserSpec {
    pub name: String,
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct Services {
    #[serde(default)]
    pub user: Vec<String>,
    #[serde(default)]
    pub system: Vec<String>,
}

#[derive(Deserialize, Default)]
pub struct ModulesConfig {
    #[serde(default)]
    pub sysctl: bool,
    #[serde(default)]
    pub modprobe: bool,
    #[serde(default)]
    pub udev: bool,
    #[serde(default)]
    pub fonts: bool,
    #[serde(default)]
    pub systemd_user: bool,
    #[serde(default)]
    pub environment: bool,
    #[serde(default)]
    pub sentinel: bool,
    #[serde(default)]
    pub etc: bool,
    #[serde(default)]
    pub systemd_system: bool,
    #[serde(default)]
    pub systemd_etc: bool,
    #[serde(default)]
    pub samba: bool,
    #[serde(default)]
    pub udev_hwdb: bool,
    #[serde(default)]
    pub appfw: bool,
    #[serde(default)]
    pub authd: bool,
    #[serde(default)]
    pub config_guard: bool,
    #[serde(default)]
    pub enpass: bool,
}

impl SetupConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read setup config: {}", path.display()))?;
        serde_yaml::from_str(&content).context("Failed to parse setup.yaml")
    }
}
