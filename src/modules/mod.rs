use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::config::SetupConfig;
use crate::utils::{color, expand_path, run_command};

/// Record of every file the modules have placed, so pruning can remove our own
/// stale files without ever touching package files or hand-edits.
const MANIFEST: &str = "/var/lib/dotfiles/placed";

/// Method for applying module files
#[derive(Clone, Copy)]
pub enum Method {
    Copy,
    Symlink,
}

/// A system module that knows how to apply configuration files
pub struct Module {
    pub name: &'static str,
    pub source_subdir: &'static str,
    pub dest_dir: &'static str,
    pub method: Method,
    pub post_hook: Option<(&'static str, &'static [&'static str])>,
    pub needs_sudo: bool,
}

impl Module {
    fn source_dir(&self, base: &Path) -> std::path::PathBuf {
        base.join("system").join(self.source_subdir)
    }

    fn dest_dir(&self) -> Result<std::path::PathBuf> {
        expand_path(self.dest_dir)
    }

    fn has_files(&self, base: &Path) -> bool {
        let src = self.source_dir(base);
        src.exists()
            && src
                .read_dir()
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
    }

    /// Apply the module and return the destination paths it placed (used to
    /// build the prune manifest). In dry-run, returns what it *would* place.
    fn apply(&self, base: &Path, dry_run: bool) -> Result<Vec<PathBuf>> {
        let src_dir = self.source_dir(base);
        if !src_dir.exists() {
            return Ok(Vec::new());
        }

        let dest_dir = self.dest_dir()?;
        self.ensure_dest_dir(&dest_dir, dry_run)?;

        let mut placed = Vec::new();
        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest_dir.join(&file_name);
            self.apply_entry(&src_path, &dest_path, dry_run)?;
            placed.push(dest_path);
        }

        self.run_post_hook(dry_run)?;
        Ok(placed)
    }

    fn ensure_dest_dir(&self, dest_dir: &Path, dry_run: bool) -> Result<()> {
        if dest_dir.exists() {
            return Ok(());
        }

        if dry_run {
            println!("  Would create directory: {}", dest_dir.display());
            return Ok(());
        }

        if self.needs_sudo {
            run_command("mkdir", &["-p", &dest_dir.to_string_lossy()], true)?;
        } else {
            fs::create_dir_all(dest_dir)?;
        }

        Ok(())
    }

    fn apply_entry(&self, src_path: &Path, dest_path: &Path, dry_run: bool) -> Result<()> {
        match self.method {
            Method::Copy => self.copy_entry(src_path, dest_path, dry_run),
            Method::Symlink => self.symlink_entry(src_path, dest_path, dry_run),
        }
    }

    fn copy_entry(&self, src_path: &Path, dest_path: &Path, dry_run: bool) -> Result<()> {
        if dry_run {
            println!(
                "  Would copy: {} -> {}",
                src_path.display(),
                dest_path.display()
            );
            return Ok(());
        }

        if self.needs_sudo {
            run_command(
                "cp",
                &[&src_path.to_string_lossy(), &dest_path.to_string_lossy()],
                true,
            )?;
        } else {
            fs::copy(src_path, dest_path)
                .with_context(|| format!("Failed to copy {}", src_path.display()))?;
        }

        println!(
            "  Copied: {} -> {}",
            src_path.display(),
            dest_path.display()
        );
        Ok(())
    }

    fn symlink_entry(&self, src_path: &Path, dest_path: &Path, dry_run: bool) -> Result<()> {
        self.remove_existing_dest(dest_path, dry_run)?;

        if dry_run {
            println!(
                "  Would link: {} -> {}",
                dest_path.display(),
                src_path.display()
            );
            return Ok(());
        }

        if self.needs_sudo {
            run_command(
                "ln",
                &[
                    "-s",
                    &src_path.to_string_lossy(),
                    &dest_path.to_string_lossy(),
                ],
                true,
            )?;
        } else {
            symlink(src_path, dest_path)?;
        }

        println!(
            "  Linked: {} -> {}",
            dest_path.display(),
            src_path.display()
        );
        Ok(())
    }

    fn remove_existing_dest(&self, dest_path: &Path, dry_run: bool) -> Result<()> {
        if dest_path.symlink_metadata().is_err() {
            return Ok(());
        }

        if dry_run {
            println!("  Would remove: {}", dest_path.display());
            return Ok(());
        }

        if self.needs_sudo {
            run_command("rm", &["-f", &dest_path.to_string_lossy()], true)?;
        } else {
            fs::remove_file(dest_path)?;
        }

        Ok(())
    }

    fn run_post_hook(&self, dry_run: bool) -> Result<()> {
        let Some((cmd, args)) = self.post_hook else {
            return Ok(());
        };

        if dry_run {
            println!("  Would run: {} {:?}", cmd, args);
            return Ok(());
        }

        println!("  Running: {} {:?}", cmd, args);
        run_command(cmd, args, self.needs_sudo)
    }
}

/// All built-in modules
const MODULES: &[Module] = &[
    Module {
        name: "sysctl",
        source_subdir: "sysctl.d",
        dest_dir: "/etc/sysctl.d",
        method: Method::Copy,
        post_hook: Some(("sysctl", &["--system"])),
        needs_sudo: true,
    },
    Module {
        name: "modprobe",
        source_subdir: "modprobe.d",
        dest_dir: "/etc/modprobe.d",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "udev",
        source_subdir: "udev.d",
        dest_dir: "/etc/udev/rules.d",
        method: Method::Copy,
        post_hook: Some(("udevadm", &["control", "--reload-rules"])),
        needs_sudo: true,
    },
    Module {
        name: "fonts",
        source_subdir: "fonts",
        dest_dir: "~/.local/share/fonts",
        method: Method::Copy,
        post_hook: Some(("fc-cache", &["-fv"])),
        needs_sudo: false,
    },
    Module {
        name: "systemd-user",
        source_subdir: "systemd/user",
        dest_dir: "~/.config/systemd/user",
        method: Method::Symlink,
        post_hook: Some(("systemctl", &["--user", "daemon-reload"])),
        needs_sudo: false,
    },
    Module {
        name: "environment",
        source_subdir: "environment.d",
        dest_dir: "~/.config/environment.d",
        method: Method::Symlink,
        post_hook: None,
        needs_sudo: false,
    },
    Module {
        name: "sentinel",
        source_subdir: "sentinel",
        dest_dir: "/etc/sentinel",
        method: Method::Symlink,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "etc",
        source_subdir: "etc",
        dest_dir: "/etc",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "systemd-system",
        source_subdir: "systemd-system",
        dest_dir: "/etc/systemd/system",
        method: Method::Copy,
        post_hook: Some(("systemctl", &["daemon-reload"])),
        needs_sudo: true,
    },
    Module {
        name: "systemd-etc",
        source_subdir: "systemd-etc",
        dest_dir: "/etc/systemd",
        method: Method::Copy,
        post_hook: Some(("systemctl", &["daemon-reload"])),
        needs_sudo: true,
    },
    Module {
        name: "samba",
        source_subdir: "samba",
        dest_dir: "/etc/samba",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "udev-hwdb",
        source_subdir: "udev-hwdb",
        dest_dir: "/etc/udev/hwdb.d",
        method: Method::Copy,
        post_hook: Some(("systemd-hwdb", &["update"])),
        needs_sudo: true,
    },
    Module {
        name: "appfw",
        source_subdir: "appfw",
        dest_dir: "/etc/appfw/rules.d",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "authd",
        source_subdir: "authd",
        dest_dir: "/etc/authd/policies.d",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "config-guard",
        source_subdir: "config-guard",
        dest_dir: "/etc/config-guard",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "enpass",
        source_subdir: "enpass",
        dest_dir: "/etc/Enpass",
        method: Method::Copy,
        post_hook: None,
        needs_sudo: true,
    },
    Module {
        name: "locale",
        source_subdir: "locale",
        dest_dir: "/etc",
        method: Method::Copy,
        post_hook: Some(("locale-gen", &[])),
        needs_sudo: true,
    },
    Module {
        name: "mkinitcpio",
        source_subdir: "mkinitcpio",
        dest_dir: "/etc",
        method: Method::Copy,
        post_hook: Some(("mkinitcpio", &["-P"])),
        needs_sudo: true,
    },
];

fn is_module_enabled(config: &SetupConfig, name: &str) -> bool {
    match name {
        "sysctl" => config.modules.sysctl,
        "modprobe" => config.modules.modprobe,
        "udev" => config.modules.udev,
        "fonts" => config.modules.fonts,
        "systemd-user" => config.modules.systemd_user,
        "environment" => config.modules.environment,
        "sentinel" => config.modules.sentinel,
        "etc" => config.modules.etc,
        "systemd-system" => config.modules.systemd_system,
        "systemd-etc" => config.modules.systemd_etc,
        "samba" => config.modules.samba,
        "udev-hwdb" => config.modules.udev_hwdb,
        "appfw" => config.modules.appfw,
        "authd" => config.modules.authd,
        "config-guard" => config.modules.config_guard,
        "enpass" => config.modules.enpass,
        "locale" => config.modules.locale,
        "mkinitcpio" => config.modules.mkinitcpio,
        _ => false,
    }
}

pub fn run_status(config: &SetupConfig, source_dir: &Path) -> Result<()> {
    println!("System modules:");
    for module in MODULES {
        let enabled = is_module_enabled(config, module.name);
        let has_files = module.has_files(source_dir);

        let status = if enabled && has_files {
            format!("{}✓{} enabled, has files", color::GREEN, color::RESET)
        } else if enabled {
            format!("{}○{} enabled, no files", color::YELLOW, color::RESET)
        } else if has_files {
            format!("{}○{} disabled, has files", color::YELLOW, color::RESET)
        } else {
            "  disabled".to_string()
        };

        println!("  {}: {}", module.name, status);
    }
    Ok(())
}

pub fn run_apply(
    config: &SetupConfig,
    source_dir: &Path,
    dry_run: bool,
    prune: bool,
) -> Result<()> {
    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    let mut applied = 0;
    let mut skipped = 0;
    let mut placed: BTreeSet<PathBuf> = BTreeSet::new();

    for module in MODULES {
        if !is_module_enabled(config, module.name) || !module.has_files(source_dir) {
            skipped += 1;
            continue;
        }

        println!("\n[{}]", module.name);
        placed.extend(module.apply(source_dir, dry_run)?);
        applied += 1;
    }

    println!("\n{applied} modules applied, {skipped} skipped");
    reconcile_manifest(&placed, dry_run, prune)?;
    Ok(())
}

/// Read the set of files we placed on a previous run.
fn load_manifest() -> BTreeSet<PathBuf> {
    fs::read_to_string(MANIFEST)
        .map(|s| {
            s.lines()
                .filter(|l| !l.trim().is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Persist the manifest (root-owned, under /var/lib) via sudo.
fn write_manifest(paths: &BTreeSet<PathBuf>) -> Result<()> {
    let body: String = paths.iter().map(|p| format!("{}\n", p.display())).collect();
    let tmp = std::env::temp_dir().join("dotfiles-placed");
    fs::write(&tmp, body).context("Failed to write temp manifest")?;
    run_command("mkdir", &["-p", "/var/lib/dotfiles"], true)?;
    run_command("cp", &[&tmp.to_string_lossy(), MANIFEST], true)?;
    let _ = fs::remove_file(&tmp);
    Ok(())
}

/// Compare what we just placed against the manifest. Files we placed before but
/// no longer produce are "stale"; with --prune they're removed (only ever files
/// in our own manifest — never package files or hand-edits). Unpruned stale
/// stays tracked so a later prune can still catch it.
fn reconcile_manifest(placed: &BTreeSet<PathBuf>, dry_run: bool, prune: bool) -> Result<()> {
    let previous = load_manifest();
    let stale: Vec<PathBuf> = previous
        .iter()
        .filter(|p| !placed.contains(*p))
        .cloned()
        .collect();

    for path in &stale {
        if !prune {
            println!("  Stale (run --prune to remove): {}", path.display());
        } else if dry_run {
            println!("  Would prune /etc orphan: {}", path.display());
        } else if path.exists() {
            run_command("rm", &["-f", &path.to_string_lossy()], true)?;
            println!("  Pruned /etc orphan: {}", path.display());
        }
    }

    if dry_run {
        return Ok(());
    }

    let mut manifest = placed.clone();
    if !prune {
        manifest.extend(stale); // keep unpruned orphans tracked
    }
    write_manifest(&manifest)
}
