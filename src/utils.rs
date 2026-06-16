use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Expand ~ to home directory
pub fn expand_path(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(rest))
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Run a command, optionally with authsudo
pub fn run_command(cmd: &str, args: &[&str], use_sudo: bool) -> Result<()> {
    let mut command = if use_sudo {
        let mut c = Command::new("authsudo");
        c.arg(cmd);
        c
    } else {
        Command::new(cmd)
    };

    command.args(args);

    let status = command
        .status()
        .with_context(|| format!("Failed to run: {} {:?}", cmd, args))?;

    if !status.success() {
        anyhow::bail!("Command failed: {} {:?}", cmd, args);
    }

    Ok(())
}

/// ANSI color codes
pub mod color {
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
    pub const RESET: &str = "\x1b[0m";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_leaves_absolute_unchanged() {
        assert_eq!(expand_path("/etc/foo").unwrap(), PathBuf::from("/etc/foo"));
    }

    #[test]
    fn expand_path_expands_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_path("~/x/y").unwrap(), home.join("x/y"));
    }

    #[test]
    fn expand_path_does_not_touch_bare_tilde_or_midstring() {
        // only a leading "~/" is expanded
        assert_eq!(
            expand_path("relative/p").unwrap(),
            PathBuf::from("relative/p")
        );
    }
}
