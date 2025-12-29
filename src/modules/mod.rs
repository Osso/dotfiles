use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::config::SetupConfig;
use crate::utils::{color, expand_path, run_command};

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
        src.exists() && src.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false)
    }

    fn apply(&self, base: &Path, dry_run: bool) -> Result<()> {
        let src_dir = self.source_dir(base);
        let dest_dir = self.dest_dir()?;

        if !src_dir.exists() {
            return Ok(());
        }

        // Ensure dest dir exists
        if !dest_dir.exists() {
            if dry_run {
                println!("  Would create directory: {}", dest_dir.display());
            } else if self.needs_sudo {
                run_command("mkdir", &["-p", &dest_dir.to_string_lossy()], true)?;
            } else {
                fs::create_dir_all(&dest_dir)?;
            }
        }

        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest_dir.join(&file_name);

            match self.method {
                Method::Copy => {
                    if dry_run {
                        println!("  Would copy: {} -> {}", src_path.display(), dest_path.display());
                    } else if self.needs_sudo {
                        run_command(
                            "cp",
                            &[&src_path.to_string_lossy(), &dest_path.to_string_lossy()],
                            true,
                        )?;
                        println!("  Copied: {} -> {}", src_path.display(), dest_path.display());
                    } else {
                        fs::copy(&src_path, &dest_path)
                            .with_context(|| format!("Failed to copy {}", src_path.display()))?;
                        println!("  Copied: {} -> {}", src_path.display(), dest_path.display());
                    }
                }
                Method::Symlink => {
                    // Remove existing if present
                    if dest_path.symlink_metadata().is_ok() {
                        if dry_run {
                            println!("  Would remove: {}", dest_path.display());
                        } else if self.needs_sudo {
                            run_command("rm", &["-f", &dest_path.to_string_lossy()], true)?;
                        } else {
                            fs::remove_file(&dest_path)?;
                        }
                    }

                    if dry_run {
                        println!("  Would link: {} -> {}", dest_path.display(), src_path.display());
                    } else if self.needs_sudo {
                        run_command(
                            "ln",
                            &["-s", &src_path.to_string_lossy(), &dest_path.to_string_lossy()],
                            true,
                        )?;
                        println!("  Linked: {} -> {}", dest_path.display(), src_path.display());
                    } else {
                        symlink(&src_path, &dest_path)?;
                        println!("  Linked: {} -> {}", dest_path.display(), src_path.display());
                    }
                }
            }
        }

        // Run post-hook if any
        if let Some((cmd, args)) = self.post_hook {
            if dry_run {
                println!("  Would run: {} {:?}", cmd, args);
            } else {
                println!("  Running: {} {:?}", cmd, args);
                run_command(cmd, args, self.needs_sudo)?;
            }
        }

        Ok(())
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
            format!("  disabled")
        };

        println!("  {}: {}", module.name, status);
    }
    Ok(())
}

pub fn run_apply(config: &SetupConfig, source_dir: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    let mut applied = 0;
    let mut skipped = 0;

    for module in MODULES {
        if !is_module_enabled(config, module.name) {
            skipped += 1;
            continue;
        }

        if !module.has_files(source_dir) {
            skipped += 1;
            continue;
        }

        println!("\n[{}]", module.name);
        module.apply(source_dir, dry_run)?;
        applied += 1;
    }

    println!("\n{applied} modules applied, {skipped} skipped");
    Ok(())
}
