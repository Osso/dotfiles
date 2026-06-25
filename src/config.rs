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
    #[serde(default)]
    pub timezone: Option<String>,
    /// Number of pre-apply root snapshots ("generations") to keep. 0 = disabled.
    #[serde(default)]
    pub generations: u32,
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
    #[serde(default)]
    pub locale: bool,
    #[serde(default)]
    pub mkinitcpio: bool,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("dotfiles-config-{tag}-{}-{n}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn enabled_modules(config: &ModulesConfig) -> Vec<&'static str> {
        let module_flags = [
            ("sysctl", config.sysctl),
            ("modprobe", config.modprobe),
            ("udev", config.udev),
            ("fonts", config.fonts),
            ("systemd-user", config.systemd_user),
            ("environment", config.environment),
            ("sentinel", config.sentinel),
            ("etc", config.etc),
            ("systemd-system", config.systemd_system),
            ("systemd-etc", config.systemd_etc),
            ("samba", config.samba),
            ("udev-hwdb", config.udev_hwdb),
            ("appfw", config.appfw),
            ("authd", config.authd),
            ("config-guard", config.config_guard),
            ("enpass", config.enpass),
            ("locale", config.locale),
            ("mkinitcpio", config.mkinitcpio),
        ];

        module_flags
            .into_iter()
            .filter_map(|(name, enabled)| enabled.then_some(name))
            .collect()
    }

    #[test]
    fn links_config_loads_explicit_and_pattern_entries() {
        let root = temp_dir("links");
        let config_path = root.join("config.toml");
        fs::write(
            &config_path,
            r#"
source_dir = "/tmp/source"
exclude = ["config/private"]

[links]
"git/.gitconfig" = "~/.gitconfig"

[patterns]
"config/*" = "~/.config/*"
"#,
        )
        .unwrap();

        let config = LinksConfig::load(config_path.to_str().unwrap()).unwrap();

        assert_eq!(config.source_dir, "/tmp/source");
        assert_eq!(
            config.links.get("git/.gitconfig").map(String::as_str),
            Some("~/.gitconfig")
        );
        assert_eq!(
            config.patterns.get("config/*").map(String::as_str),
            Some("~/.config/*")
        );
        assert_eq!(config.exclude, vec!["config/private"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn links_config_defaults_optional_collections() {
        let root = temp_dir("links-defaults");
        let config_path = root.join("config.toml");
        fs::write(&config_path, r#"source_dir = "/tmp/source""#).unwrap();

        let config = LinksConfig::load(config_path.to_str().unwrap()).unwrap();

        assert!(config.links.is_empty());
        assert!(config.patterns.is_empty());
        assert!(config.exclude.is_empty());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn links_config_reports_missing_file() {
        let root = temp_dir("links-missing");
        let config_path = root.join("missing.toml");

        let error = match LinksConfig::load(config_path.to_str().unwrap()) {
            Ok(_) => panic!("missing links config should fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("Failed to read config"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn setup_config_missing_file_is_default() {
        let root = temp_dir("setup-missing");
        let config = SetupConfig::load(&root.join("setup.yaml")).unwrap();

        assert!(config.services.user.is_empty());
        assert!(config.services.system.is_empty());
        assert!(config.directories.is_empty());
        assert!(config.users.is_empty());
        assert_eq!(config.timezone, None);
        assert_eq!(config.generations, 0);
        assert!(enabled_modules(&config.modules).is_empty());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn setup_config_loads_nested_sections() {
        let root = temp_dir("setup");
        let config_path = root.join("setup.yaml");
        fs::write(
            &config_path,
            r#"
services:
  user: ["syncthing.service"]
  system: ["sshd.service"]
directories:
  - "~/.local/bin"
timezone: "America/Chicago"
generations: 7
modules:
  fonts: true
  systemd_user: true
users:
  - name: "alice"
    shell: "/bin/zsh"
    groups: ["wheel", "docker"]
"#,
        )
        .unwrap();

        let config = SetupConfig::load(&config_path).unwrap();

        assert_eq!(config.services.user, vec!["syncthing.service"]);
        assert_eq!(config.services.system, vec!["sshd.service"]);
        assert_eq!(config.directories, vec!["~/.local/bin"]);
        assert_eq!(config.timezone.as_deref(), Some("America/Chicago"));
        assert_eq!(config.generations, 7);
        assert_eq!(
            enabled_modules(&config.modules),
            vec!["fonts", "systemd-user"]
        );
        assert_eq!(config.users[0].name, "alice");
        assert_eq!(config.users[0].shell.as_deref(), Some("/bin/zsh"));
        assert_eq!(config.users[0].groups, vec!["wheel", "docker"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn setup_config_reports_parse_errors() {
        let root = temp_dir("setup-invalid");
        let config_path = root.join("setup.yaml");
        fs::write(&config_path, "services: [").unwrap();

        let error = match SetupConfig::load(&config_path) {
            Ok(_) => panic!("invalid setup config should fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("Failed to parse setup.yaml"));
        fs::remove_dir_all(root).ok();
    }
}
