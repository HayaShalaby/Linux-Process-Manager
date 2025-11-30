#!/bin/bash
# Script to create a zombie process for testing
# A zombie is created when a child process exits but parent doesn't wait for it

# Disable automatic child reaping (SIGCHLD handler)
# This prevents bash from automatically reaping zombie children
trap '' SIGCHLD

# Fork a child process that exits immediately
# Parent sleeps without calling wait(), leaving child as zombie
(
    # Child process - exits immediately (becomes zombie)
    exit 0
) &

# Store the child PID
CHILD_PID=$!

# Give it a moment to exit and become a zombie
sleep 0.1

# Verify it's a zombie
if ps -p $CHILD_PID -o stat= 2>/dev/null | grep -q Z; then
    echo "✓ Successfully created zombie process!"
    echo "  PID: $CHILD_PID"
    echo "  State: Z (Zombie)"
    echo ""
    echo "Check it in Process Manager:"
    echo "  1. Refresh the process list"
    echo "  2. Search for PID: $CHILD_PID"
    echo "  3. Or enable: View → Show Only Zombie Processes"
    echo ""
    echo "The zombie will remain until:"
    echo "  - This script exits (Ctrl+C)"
    echo "  - This script calls 'wait' or 'wait -n'"
    echo ""
    echo "Press Ctrl+C to exit and clean up the zombie"
else
    echo "⚠ Warning: Process $CHILD_PID is not a zombie"
    echo "  Current state: $(ps -p $CHILD_PID -o stat= 2>/dev/null || echo 'not found')"
    echo "  This might be because bash reaped it automatically"
fi

# Parent process sleeps, not calling wait()
# The child remains a zombie
sleep 60
