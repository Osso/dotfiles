#!/bin/bash
# Sudo askpass with keyring caching
# First time: asks for password and stores in keyring
# Subsequent: shows accept/deny dialog

KEYRING_ATTR="application"
KEYRING_VAL="sudo-askpass"

# Try to get cached password
cached=$(secret-tool lookup $KEYRING_ATTR $KEYRING_VAL 2>/dev/null)

if [[ -n "$cached" ]]; then
    # Password cached - show confirm dialog
    if zenity --question --text="Allow sudo?" --title="Sudo" --timeout=30 2>/dev/null; then
        echo "$cached"
    else
        exit 1
    fi
else
    # No cache - prompt for password
    password=$(zenity --password --title="Sudo password (will be cached)" 2>/dev/null)
    if [[ -n "$password" ]]; then
        # Store in keyring
        echo "$password" | secret-tool store --label="sudo password" $KEYRING_ATTR $KEYRING_VAL 2>/dev/null
        echo "$password"
    else
        exit 1
    fi
fi