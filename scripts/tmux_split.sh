#!/usr/bin/env bash

# This script splits the current tmux window and runs boot.sh in the lower pane.

# Check if already in tmux
if [ -z "$TMUX" ]; then
  echo "Not in a tmux session. Please start a tmux session first."
  exit 1
fi

# Split the current window horizontally
tmux split-window -h

# Navigate to the newly created lower pane
tmux select-pane -D

# Send commands to the lower pane to run boot.sh
tmux send-keys "cd $(dirname "$(dirname "$0")") && ./boot.sh" C-m

echo "Tmux window split and boot.sh launched in the lower pane."
