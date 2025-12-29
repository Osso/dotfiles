#!/bin/bash
# Focus Firefox window, or launch if not running
id=$(niri msg -j windows | jq -r '.[] | select(.app_id == "firefox") | .id' | head -1)
if [[ -n "$id" ]]; then
    niri msg action focus-window --id "$id"
else
    firefox &
fi
