#!/usr/bin/env bash

# This boot.sh is tailored for the submodule to launch the Gemini CLI.

# Configuration
SUBMODULE_NAME=$(basename "$(pwd)")
SESSION_NAME="crq-${CRQ_NUMBER}-${SUBMODULE_NAME}"
LOG_DIR=".gemini_logs"
RECORDING_DIR="${LOG_DIR}/recordings"
mkdir -p "$RECORDING_DIR"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Asciinema recording
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
ASCIINEMA_REC_FILE="${RECORDING_DIR}/session_${TIMESTAMP}.cast"

echo "Starting asciinema recording to: ${ASCIINEMA_REC_FILE}"

# The command to be executed inside tmux, which launches the Gemini CLI
TMUX_INNER_COMMAND="nix develop --command bash -c \"/data/data/com.termux.nix/files/home/pick-up-nix2/gemini_cli_recent.sh\""

# Start asciinema recording, and inside it, start/attach to a tmux session.
# The tmux session will then execute the gemini command.
# The 'bash -c' is used to ensure the inner command is executed correctly within tmux.
ascinema rec "${ASCIINEMA_REC_FILE}" --command "tmux new-session -A -s \"${SESSION_NAME}\" \; send-keys -t \"${SESSION_NAME}\" \"${TMUX_INNER_COMMAND}\" C-m"

echo "Recording finished. To play: asciinema play ${ASCIINEMA_REC_FILE}"

# Initiate Crash Recovery Checks (adjusted for submodule context)
echo "--- Initiating Crash Recovery Checks ---" | tee -a "$LOG_DIR/crash_recovery_log.txt"
echo "Git Status:" | tee -a "$LOG_DIR/crash_recovery_log.txt"
git status --ignore-submodules | tee -a "$LOG_DIR/crash_recovery_log.txt"
echo "" | tee -a "$LOG_DIR/crash_recovery_log.txt"

echo "Git Diff HEAD:" | tee -a "$LOG_DIR/crash_recovery_log.txt"
git diff HEAD | tee -a "$LOG_DIR/crash_recovery_log.txt"
echo "" | tee -a "$LOG_DIR/crash_recovery_log.txt"

echo "--- Crash Recovery Checks Complete ---" | tee -a "$LOG_DIR/crash_recovery_log.txt"

