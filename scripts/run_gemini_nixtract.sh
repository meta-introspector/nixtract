#!/usr/bin/env bash

# This script is for running Gemini CLI commands within the nixtract submodule context.

# Navigate to the main project root and source its boot.sh
# Assuming the main project root is two levels up from this script:
# pick-up-nix2/vendor/nix/nixtract/scripts/run_gemini_nixtract.sh
# So, main project root is ../../../
MAIN_PROJECT_ROOT="$(dirname "$(dirname "$(dirname "$(realpath "$0")")")")"
source "$MAIN_PROJECT_ROOT/boot.sh"

echo "Gemini CLI environment for nixtract submodule is set up."
echo "You can now run Gemini CLI commands, e.g.:"
echo "gemini_cli.sh --task-file <path_to_nixtract_task_file.md>"

# Example: If you want to run a specific Gemini CLI command directly:
# "$MAIN_PROJECT_ROOT/gemini_cli.sh" --task-file "$MAIN_PROJECT_ROOT/task/nixtract_specific_task.md"
