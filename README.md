# dotfiles

Personal dotfiles for Linux (Arch/Debian).

## What's included

### Terminal emulators
- **kitty** - GPU-accelerated terminal with custom theme
- **wezterm** - Cross-platform terminal with Lua config

### Shells
- **bash/zsh** - Shared configuration for both shells
  - Prompt (starship)
  - History settings
  - Aliases and functions
  - Environment variables
  - Key bindings

### Other configs
- **git** - Aliases, colors, merge settings
- **starship** - Cross-shell prompt
- **nano** - Basic editor config

### Utility scripts (`bin/`)

| Script | Description |
|--------|-------------|
| `firefox-tabs` | List/search open Firefox tabs |
| `elgato-light.py` | Control Elgato Key Light |
| `usb-mount` | Mount USB drives with udisks2 |
| `docker-cleanup` | Clean up Docker resources |
| `kube-top.sh` | Kubernetes resource usage |
| `git-delete-branch` | Delete local and remote branch |
| `git-strip-diff` | Strip diff headers |
| `diff-highlight` | Highlight diff changes |
| `kitty-remote` | Remote control kitty terminal |
| `wezterm-remote` | Remote control wezterm terminal |
| `clip2path` | Save clipboard image to file |
| `focus-firefox.sh` | Focus Firefox window |
| `fix-keyboard.sh` | Reset keyboard settings |
| `high-dpi-scaling.sh` | Configure HiDPI scaling |
| `sudo-askpass.sh` | GUI sudo password prompt |
| `polkit-agent-wrapper` | Polkit authentication wrapper |

## Installation

The repo includes Ansible playbooks for automated setup:

```bash
# Set your dotfiles path
export PROVISIONING_PATH=~/dotfiles

# Run the playbook
ansible-playbook playbooks/playbook.yml -i 127.0.0.1, -Kv
```

Or manually symlink what you need.

## License

MIT
