#!/usr/bin/env bash

# This script is used to boot the development environment for nixtract.
# It is designed to be sourced by other scripts or run directly.

# Get the directory of the current script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# --- Dependency Checks ---

# Check for direnv
if ! command -v direnv &> /dev/null; then
    echo "Error: direnv is not installed or not in PATH." >&2
    echo "Please install direnv to proceed: https://direnv.net/docs/installation.html" >&2
    exit 1
fi

# Check for nix
if ! command -v nix &> /dev/null; then
    echo "Error: Nix is not installed or not in PATH. Nix development environment cannot be entered." >&2
    exit 1
fi

# --- Environment Setup ---

# Allow direnv to load the environment variables from .envrc in the script's directory
# This ensures direnv is always applied to the correct project root.
direnv allow "${SCRIPT_DIR}"

# Use nix develop to enter the development environment
echo "Entering Nix development environment for nixtract..."
nix develop "${SCRIPT_DIR}" --command bash -c "echo 'Welcome to the nixtract development environment! Exited Nix development environment.'"

# Uncomment the following line if you want to use nix-shell instead
# nix-shell "${SCRIPT_DIR}" --command bash -c "echo 'Welcome to the nixtract development environment! Exited Nix shell environment.'"

