#[cfg(not(coverage))]
use anyhow::{Context, Result};
#[cfg(not(coverage))]
use std::process::Command;

#[cfg(not(coverage))]
use crate::config::{SetupConfig, UserSpec};
#[cfg(not(coverage))]
use crate::utils::run_command;

/// Current state of an existing user, as read from the system.
#[cfg(not(coverage))]
struct UserInfo {
    shell: String,
    groups: Vec<String>,
}

#[cfg(not(coverage))]
pub fn run_users(config: &SetupConfig, dry_run: bool) -> Result<()> {
    if config.users.is_empty() {
        return Ok(());
    }

    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    println!("Users:");
    for user in &config.users {
        ensure_groups(&user.groups, dry_run)?;
        match user_info(&user.name)? {
            None => create_user(user, dry_run)?,
            Some(info) => reconcile_user(user, &info, dry_run)?,
        }
    }
    Ok(())
}

/// Create any referenced groups that don't exist yet, so the user's group
/// memberships can be applied on a fresh machine.
#[cfg(not(coverage))]
fn ensure_groups(groups: &[String], dry_run: bool) -> Result<()> {
    for group in groups {
        if group_exists(group)? {
            continue;
        }
        if dry_run {
            println!("  Would create group: {}", group);
        } else {
            println!("  Creating group: {}", group);
            run_command("groupadd", &[group], true)?;
        }
    }
    Ok(())
}

#[cfg(not(coverage))]
fn group_exists(name: &str) -> Result<bool> {
    Ok(Command::new("getent")
        .args(["group", name])
        .output()
        .context("Failed to run getent")?
        .status
        .success())
}

/// Read a user's shell and group membership, or `None` if it doesn't exist.
#[cfg(not(coverage))]
fn user_info(name: &str) -> Result<Option<UserInfo>> {
    let passwd = Command::new("getent")
        .args(["passwd", name])
        .output()
        .context("Failed to run getent")?;
    if !passwd.status.success() {
        return Ok(None);
    }

    let line = String::from_utf8_lossy(&passwd.stdout);
    let shell = line.trim().split(':').nth(6).unwrap_or("").to_string();

    let groups_out = Command::new("id")
        .args(["-nG", name])
        .output()
        .context("Failed to run id")?;
    let groups = String::from_utf8_lossy(&groups_out.stdout)
        .split_whitespace()
        .map(String::from)
        .collect();

    Ok(Some(UserInfo { shell, groups }))
}

#[cfg(not(coverage))]
fn create_user(user: &UserSpec, dry_run: bool) -> Result<()> {
    let mut args = vec!["-m".to_string()];
    if let Some(shell) = &user.shell {
        args.push("-s".into());
        args.push(shell.clone());
    }
    if !user.groups.is_empty() {
        args.push("-G".into());
        args.push(user.groups.join(","));
    }
    args.push(user.name.clone());

    if dry_run {
        println!("  Would create user: useradd {}", args.join(" "));
        return Ok(());
    }

    println!("  Creating user: {}", user.name);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_command("useradd", &arg_refs, true)
}

/// Bring an existing user in line with the spec: fix the shell and add any
/// missing group memberships. Never removes the user or strips group
/// memberships — undeclared supplementary groups are reported, not touched.
#[cfg(not(coverage))]
fn reconcile_user(user: &UserSpec, info: &UserInfo, dry_run: bool) -> Result<()> {
    let mut changed = false;

    if let Some(shell) = &user.shell
        && &info.shell != shell
    {
        changed = true;
        if dry_run {
            println!(
                "  Would set {} shell: {} -> {}",
                user.name, info.shell, shell
            );
        } else {
            println!("  Setting {} shell: {}", user.name, shell);
            run_command("usermod", &["-s", shell, &user.name], true)?;
        }
    }

    let missing = missing_groups(&user.groups, &info.groups);
    if !missing.is_empty() {
        changed = true;
        let add = missing.join(",");
        if dry_run {
            println!("  Would add {} to groups: {}", user.name, add);
        } else {
            println!("  Adding {} to groups: {}", user.name, add);
            run_command("usermod", &["-aG", &add, &user.name], true)?;
        }
    }

    let extra = undeclared_groups(&user.groups, &info.groups, &user.name);
    if !extra.is_empty() {
        println!(
            "  Note: {} also in undeclared groups (not removed): {}",
            user.name,
            extra.join(", ")
        );
    }

    if !changed {
        println!("  {} (ok)", user.name);
    }
    Ok(())
}

/// Declared groups the user isn't a member of yet (to add).
fn missing_groups<'a>(declared: &'a [String], current: &[String]) -> Vec<&'a str> {
    declared
        .iter()
        .filter(|g| !current.contains(g))
        .map(String::as_str)
        .collect()
}

/// Current supplementary groups that aren't declared — reported, never removed.
/// The primary group (same name as the user) is ignored.
fn undeclared_groups<'a>(
    declared: &[String],
    current: &'a [String],
    primary: &str,
) -> Vec<&'a str> {
    current
        .iter()
        .filter(|g| g.as_str() != primary && !declared.contains(g))
        .map(String::as_str)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn missing_groups_are_declared_minus_current() {
        let declared = s(&["wheel", "docker", "video"]);
        let current = s(&["osso", "wheel"]);
        assert_eq!(missing_groups(&declared, &current), vec!["docker", "video"]);
    }

    #[test]
    fn missing_groups_empty_when_all_present() {
        let declared = s(&["wheel"]);
        let current = s(&["osso", "wheel", "docker"]);
        assert!(missing_groups(&declared, &current).is_empty());
    }

    #[test]
    fn undeclared_groups_excludes_primary_and_declared() {
        let declared = s(&["wheel"]);
        let current = s(&["osso", "wheel", "docker", "input"]);
        // primary "osso" and declared "wheel" excluded; docker+input reported
        assert_eq!(
            undeclared_groups(&declared, &current, "osso"),
            vec!["docker", "input"]
        );
    }
}
