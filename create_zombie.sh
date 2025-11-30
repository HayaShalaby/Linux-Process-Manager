#!/bin/bash
# Script to create a zombie process for testing
# A zombie is created when a child process exits but parent doesn't wait for it

# Fork a child process that exits immediately
# Parent sleeps without calling wait(), leaving child as zombie
(
    # Child process - exits immediately
    exit 0
) &

# Parent process sleeps, not calling wait()
# The child becomes a zombie
echo "Created zombie process. PID: $!"
echo "The child process will be a zombie until this script exits or calls wait"
echo "Press Ctrl+C to exit and clean up the zombie"
sleep 60

