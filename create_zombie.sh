#!/bin/bash
# Script to create a zombie process for testing
# A zombie is created when a child process exits but parent doesn't wait for it

# Fork a child process that exits immediately
# Parent sleeps without calling wait(), leaving child as zombie
(
    # Child process - exits immediately (becomes zombie)
    exit 0
) &

# Store the child PID
CHILD_PID=$!

# Parent process sleeps, not calling wait()
# The child becomes a zombie immediately after exiting
echo "Created zombie process. PID: $CHILD_PID"
echo "The child process has exited and is now a ZOMBIE (state 'Z')"
echo "Check it in Process Manager - it should show state 'Z'"
echo ""
echo "The zombie will remain until:"
echo "  - This script exits (Ctrl+C)"
echo "  - This script calls 'wait'"
echo "  - System cleans it up"
echo ""
echo "Press Ctrl+C to exit and clean up the zombie"
sleep 60

