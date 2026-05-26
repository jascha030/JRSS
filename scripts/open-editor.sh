#!/usr/bin/env bash
# Launch script for Svelte Inspector -> Neovim via neovim-remote (nvr)
# Usage: LAUNCH_EDITOR=scripts/open-editor.sh bun run tauri:dev
#
# Requires nvim to be running with a listen socket. Start your editor with:
#   nvim --listen /tmp/nvim-socket
# Or set the env var before launching nvim:
#   NVIM_LISTEN_ADDRESS=/tmp/nvim-socket nvim

set -e

filename="$1"
line="${2:-1}"
column="${3:-1}"

socket="${NVIM_LISTEN_ADDRESS:-/tmp/nvim-socket}"

if command -v nvr >/dev/null 2>&1; then
	nvr --nostart --servername "$socket" -c "edit $filename" -c "call cursor($line, $column)"
else
	echo "nvr not found. Install with: pip install neovim-remote" >&2
	exit 1
fi
