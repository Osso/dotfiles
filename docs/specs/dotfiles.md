# dotfiles — declarative system/config manager

`dotfiles` is a Rust CLI that applies a single source-of-truth provisioning repo
onto an Arch (FHS) machine: dotfile symlinks, `/etc` config, users, services,
timezone, locale, and pre-apply btrfs generation snapshots. It is the NixOS-style
declarative layer without the Nix store — config stays in standard FHS paths.

Source lives in `src/`. The data it applies lives in a separate provisioning repo
(`source_dir` in `~/.config/dotfiles/config.toml`, currently
`/syncthing/Sync/Provisioning`): `config/` (dotfiles), `system/<module>/` (`/etc`),
`setup.yaml` (modules/services/users/timezone/generations), `packages/` (pacdef).

> **Test status:** there are no automated tests yet. Every bullet in §What it must
> do is therefore `- [ ]` (contract, manually exercised via dry-runs and a live
> snapshot/prune test, but not asserted by a committed test). See §Known gaps.

## What it must do

### Links (dotfiles → `~`)
- [ ] `status` reports each declared link as ok / missing / wrong-target / not-a-symlink / broken
- [ ] `apply` creates missing/wrong symlinks, skips correct ones, never clobbers a real (non-symlink) file
- [ ] `check` exits non-zero if any declared link is incorrect
- [ ] pattern `config/* → ~/.config/*` expands per-subdirectory; `exclude` and explicit `links` are honored
- [ ] `apply --prune` removes only symlinks whose target resolves **inside** `source_dir` and are no longer declared; never touches real files or foreign symlinks

### System modules (`/etc` and user dirs)
- [ ] each enabled module copies/symlinks `system/<name>/*` to its destination, running its post-hook (e.g. `sysctl --system`, `mkinitcpio -P`, `locale-gen`)
- [ ] a module with no source files or disabled in `setup.yaml` is skipped
- [ ] `system --prune` removes files recorded in the manifest (`/var/lib/dotfiles/placed`) that are no longer produced — never package files or hand-edits
- [ ] unpruned stale entries stay tracked across runs (manifest = placed ∪ unpruned-stale)

### Users
- [ ] ensures each declared user exists with the right login shell and supplementary groups
- [ ] creates referenced groups that don't exist before applying membership
- [ ] additive only: adds missing memberships, reports-but-never-removes undeclared groups, never deletes a user

### Timezone / generations
- [ ] `timezone` points `/etc/localtime` at the declared zone (symlink, chroot-safe); reports `(ok)` when already correct
- [ ] `snapshot` takes a read-only `@arch` generation into `@snapshots/gen-<ts>` and prunes to the newest N (`generations:`, 0 = off)
- [ ] generation auto-prune is `gen-*` scoped — never deletes manual `@arch-*` snapshots
- [ ] `setup` takes a generation snapshot before applying anything

### Services / orchestration
- [ ] `services` enables the declared user + system systemd units
- [ ] `setup` runs in order: generation → users → links → system modules → timezone → services
- [ ] every mutating subcommand supports `-n/--dry-run` and makes no changes under it

## How it works

- `docs/wiki/systems/dotfiles.md` — engine architecture (stub; not yet written)
- `../../BOOTSTRAP.md` (in the provisioning repo) — bare-metal bootstrap → engine handoff

## Implementation inventory

- `src/main.rs` — clap CLI, command dispatch, `setup` orchestration order
- `src/config.rs` — `LinksConfig` (config.toml) + `SetupConfig` (setup.yaml) types
- `src/links.rs` — dotfile symlink status/apply/check + orphan pruning
- `src/modules/mod.rs` — `/etc`/user modules (copy/symlink + post-hooks) + manifest-tracked prune
- `src/users.rs` — declarative users: shell, groups, group creation
- `src/timezone.rs` — `/etc/localtime` symlink
- `src/generations.rs` — btrfs generation snapshots + keep-N prune
- `src/services.rs` — systemd unit enablement + directory creation
- `src/utils.rs` — `~` expansion, sudo (`authsudo`) command runner, ANSI colors

## Tests asserting this spec

- None yet. (No `tests/` dir, no `#[test]`, no dev-dependencies.)

## Known gaps (current cycle)

- [ ] No automated tests — add a `tests/` harness (tempdir-based) covering link expand/apply/prune, manifest prune set-math, users reconcile, and generation prune slicing. Flip §What-it-must-do bullets to `[x]` as tests land.
- [ ] Rollback is guided, not automated — `generations` prints the reboot + subvol-swap procedure; no `dotfiles rollback` command.
- [ ] `setup` is not atomic — a mid-apply failure leaves a partial state; the pre-apply generation is the recovery path, not transactional apply.

## Out of scope

- **Atomic/transactional apply** — impossible without a generated `/etc` (Nix store model); FHS-in-place was a deliberate choice. Mitigated by pre-apply generations.
- **Package management** — owned by `pacdef` (`packages/` groups), not this engine.
- **Secrets** (ssh/wifi keys, Enpass vault) and bulk data (`/home`, `/var/lib`) — restored from backup, never from the repo.
- **Partitioning / bootloader install** — documented in `BOOTSTRAP.md`, not automated (dual-boot, shared-ESP risk).
