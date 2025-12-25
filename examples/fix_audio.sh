#!/bin/bash

# 1. Find the likely active monitor source (looking for 'monitor' and 'RUNNING')
TARGET_SOURCE=$(pactl list short sources | grep "monitor" | grep "RUNNING" | awk '{print $1}' | head -n 1)

# Fallback: if no RUNNING monitor, take the first monitor found
if [ -z "$TARGET_SOURCE" ]; then
    TARGET_SOURCE=$(pactl list short sources | grep "monitor" | awk '{print $1}' | head -n 1)
fi

if [ -z "$TARGET_SOURCE" ]; then
    echo "Error: Could not find any Monitor source (System Audio)."
    exit 1
fi

echo "Target Monitor Source ID: $TARGET_SOURCE"

# 2. Find the audioviz application stream
# We look for "audioviz" in the client list or the stream name
# Since 'pactl list short source-outputs' gives IDs, we check details to be sure, 
# but simply moving the newest stream is often a good heuristic for a dev tool.
# Let's try to match the application name via 'pactl list source-outputs' verbose output if possible,
# or just look for the most recent one.

# A simple approach: Find the source-output that is NOT connected to the target source already.
# Or just move ALL recording streams to the monitor (a bit aggressive but effective for this context).

echo "Moving recording streams to Monitor..."

# Get all source-output IDs
pactl list short source-outputs | while read -r line; do
    STREAM_ID=$(echo "$line" | awk '{print $1}')
    echo "Moving stream $STREAM_ID to source $TARGET_SOURCE"
    pactl move-source-output "$STREAM_ID" "$TARGET_SOURCE"
done

echo "Done! Check the visualizer."
