# dotfiles

Declarative system/config manager for Linux (Arch). A small Rust CLI that
applies a single source-of-truth provisioning repo onto the machine —
symlinks, `/etc` modules, and systemd service enablement — replacing the old
Ansible playbooks.

This repo is the **engine**. The **data** (configs, scripts, package lists,
`/etc` modules) lives in the provisioning repo it points at.

## Layout

- `src/` — the `dotfiles` CLI
  - `links.rs` — dotfile symlinks (`config/*` → `~/.config/*`, explicit links)
  - `modules/` — `/etc` and user modules (sysctl, modprobe, udev, fonts,
    systemd-user, environment, sentinel): copy/symlink `system/<name>/*` to a
    destination, with post-hooks (e.g. `fc-cache`, `udevadm`)
  - `services.rs` — enable systemd user/system units
  - `config.rs` — `config.toml` (links) + `setup.yaml` (modules/services)

## Source repo

The engine reads its source from `~/.config/dotfiles/config.toml`
(`source_dir`). Current source: `/syncthing/Sync/Provisioning`.

- `config/<app>/` → linked into `~/.config/<app>` (pattern `config/* → ~/.config/*`)
- `system/<module>/` → applied to `/etc` (or user dirs) by the matching module
- `setup.yaml` → which modules are enabled, which services to enable
- `bin/` → linked to `~/bin`
- `packages/` → pacdef groups (package layer; separate tool, see below)

## Usage

```bash
dotfiles status              # show symlink state
dotfiles apply [-n]          # create/update symlinks (-n = dry run)
dotfiles check               # verify symlinks
dotfiles system [status] [-n]# apply /etc + user modules
dotfiles services [-n]       # enable systemd services
dotfiles setup [-n]          # apply + system + services
```

## License

MIT
