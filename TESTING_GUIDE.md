# Linux Process Manager - Testing Guide

## Prerequisites

1. **Build the application:**
   ```bash
   cargo build --release
   ```

2. **Run the application:**
   ```bash
   cargo run --release
   # OR
   ./target/release/lpm_backend
   ```

3. **Note:** Most operations require **Admin privileges**. You may need to run with `sudo`:
   ```bash
   sudo ./target/release/lpm_backend
   ```

## Testing Checklist

### 1. Basic Display & Navigation ✅

**Test: Process List Display**
- [ ] Application opens and shows process table
- [ ] All columns are visible: PID, Name, UID, State, CPU %, Memory (MB), Priority
- [ ] Processes are listed (should see many processes)
- [ ] Status bar shows process count

**Test: Search Functionality**
- [ ] Type a process name (e.g., "firefox", "chrome", "bash")
- [ ] List filters in real-time
- [ ] Try searching by PID (e.g., type "1" for init)
- [ ] Try searching by UID
- [ ] Clear search and verify all processes show again

**Test: Sorting**
- [ ] Click "PID" column header → processes sort by PID
- [ ] Click again → reverses order
- [ ] Click "CPU %" → sorts by CPU usage
- [ ] Click "Memory (MB)" → sorts by memory
- [ ] Click "Name" → alphabetical sort
- [ ] Verify sort indicator (yellow highlight on active column)

### 2. Process Details Panel ✅

**Test: View Process Details**
- [ ] Click on any process row
- [ ] Details panel appears at bottom
- [ ] Shows: PID, Name, UID, Parent PID, State, Memory, Priority, CPU %
- [ ] Click different processes → details update

### 3. Process Operations (Requires Admin) ⚠️

**⚠️ WARNING: These operations can affect system processes. Test carefully!**

**Test: Terminate Process (SIGTERM - Graceful)**
1. Find a safe test process (e.g., a terminal you opened)
2. Note its PID
3. Select the process
4. Click "Terminate" button
5. [ ] Success message appears
6. [ ] Process list refreshes
7. [ ] Process should disappear or show terminated state

**Test: Kill Process (SIGKILL - Force)**
1. Find a safe test process
2. Select it
3. Click "Kill" button
4. [ ] Success message appears
5. [ ] Process is forcefully terminated

**Test: Pause Process (SIGSTOP)**
1. Find a safe test process (e.g., a simple command running in terminal)
2. Select it
3. Click "Pause" button
4. [ ] Success message appears
5. [ ] Process state should change (may show as stopped)

**Test: Resume Process (SIGCONT)**
1. After pausing a process
2. Select the same process
3. Click "Resume" button
4. [ ] Success message appears
5. [ ] Process should resume

**Test: Set Priority**
1. Select a process
2. Enter a nice value in the input field (e.g., "10" or "-5")
3. Click "Apply"
4. [ ] Success message appears
5. [ ] Priority column should update (may need refresh)

**Test: Permission Errors**
- [ ] Try operations without sudo → should show "Permission denied" error
- [ ] Error message should be clear and helpful

### 4. Batch Operations (Requires Admin) ⚠️

**Test: Multi-Select**
- [ ] Check boxes next to multiple processes
- [ ] Status bar shows "Selected: X"
- [ ] Can select/deselect individual processes

**Test: Batch Kill**
1. Select 2-3 safe test processes
2. Go to **Operations → Kill Selected**
3. [ ] Success message shows count
4. [ ] Selected processes are killed
5. [ ] Selection is cleared

**Test: Batch Pause**
1. Select multiple processes
2. Go to **Operations → Pause Selected**
3. [ ] Success message appears
4. [ ] All selected processes are paused

**Test: Batch Resume**
1. After pausing multiple processes
2. Select them again
3. Go to **Operations → Resume Selected**
4. [ ] Success message appears
5. [ ] All processes resume

**Test: Clear Selection**
- [ ] Click "Clear Selection" button
- [ ] All checkboxes uncheck
- [ ] Status bar shows "Selected: 0"

### 5. Process Monitoring Features ✅

**Test: Abnormal Process Detection**
- [ ] Look for zombie processes (State = 'Z')
- [ ] Zombie processes should be highlighted in **YELLOW**
- [ ] Click on zombie → warning message in details panel

**Test: Creating Zombie Processes for Testing** 🆕
If you don't have any zombie processes, you can create one for testing:

**Method 1: Using the provided script**
```bash
# Run the create_zombie.sh script
./create_zombie.sh
# Keep it running in a terminal
# The child process will be a zombie
```

**Method 2: Manual creation**
```bash
# In a terminal, run:
(sleep 1 &) && sleep 60
# The sleep 1 process becomes a zombie while parent sleeps
```

**Method 3: Using the Process Manager**
1. Create a background process: **File → Create Process...**
   - Command: `sleep`
   - Args: `1`
   - Check "Run in background"
   - Click "Create"
2. Immediately kill the parent shell process (if possible)
3. Or use a script that doesn't wait for children

**After creating a zombie:**
- [ ] Open Process Manager
- [ ] Enable **View → Show Only Zombie Processes**
- [ ] Should see the zombie process with state 'Z'
- [ ] Process name should be highlighted in yellow
- [ ] Click on it → details panel shows "Zombie process" warning

**Test: CPU Percentage Calculation** 🆕
1. **Initial State Check:**
   - [ ] Open the application
   - [ ] Wait for first refresh cycle (2 seconds)
   - [ ] Check CPU % column - should show 0.0% for all processes initially
   - [ ] This is normal - CPU calculation needs 2 refresh cycles

2. **After Second Refresh:**
   - [ ] Wait for second auto-refresh (another 2 seconds)
   - [ ] CPU % column should now show real values (not all 0.0)
   - [ ] Most processes should show low CPU (< 5%)
   - [ ] Some processes may show 0.0% if they're idle

3. **Create High CPU Process:**
   ```bash
   # In a terminal, run this to create a CPU-intensive process:
   while true; do : ; done &
   # Note the PID that appears
   ```
   - [ ] Find the process in the manager (search for "bash" or the PID)
   - [ ] Wait for 2 refresh cycles
   - [ ] CPU % should show a high value (50-100% depending on cores)
   - [ ] Process should be highlighted in RED if it exceeds threshold

4. **Test CPU Calculation Accuracy:**
   ```bash
   # Create a controlled CPU load (uses 1 CPU core at 100%)
   stress-ng --cpu 1 --timeout 30s &
   # OR if stress-ng not available:
   yes > /dev/null &
   ```
   - [ ] Find the process in the manager
   - [ ] Wait for 2 refresh cycles
   - [ ] CPU % should be close to (100% / number_of_cores)
   - [ ] For example: 100% on 1-core system, ~50% on 2-core, ~25% on 4-core

5. **Verify CPU Updates:**
   - [ ] Watch a process with CPU usage
   - [ ] CPU % should update every refresh cycle (every 2 seconds)
   - [ ] Values should change as process activity changes

6. **Test CPU Sorting:**
   - [ ] Click "CPU %" column header
   - [ ] Processes should sort by CPU usage (highest first or lowest first)
   - [ ] Click again to reverse order
   - [ ] High CPU processes should be at top/bottom depending on sort

**Test: Process Timer/Uptime** 🆕
1. **Check Timer Column:**
   - [ ] Open the application
   - [ ] Look for "Timer" column in the process table
   - [ ] Timer should show process runtime (e.g., "1h 23m 45s", "5m 30s", "30s")
   - [ ] Timer updates automatically on each refresh

2. **Test Timer Sorting:**
   - [ ] Click on "Timer" column header
   - [ ] Processes should sort by uptime (oldest first or newest first)
   - [ ] Click again to reverse sort order

3. **Check Timer in Details Panel:**
   - [ ] Click on any process
   - [ ] In the right-side details panel, look for "Uptime:" field
   - [ ] Should show the same formatted uptime (e.g., "2h 15m 30s")

4. **Verify Timer Accuracy:**
   ```bash
   # In a terminal, start a process and note the time
   sleep 120 &
   # Note the PID
   ```
   - [ ] Find the process in the manager
   - [ ] Timer should show approximately the elapsed time
   - [ ] Wait a bit and refresh → timer should increase

**Test: Process Creation** 🆕
1. **Create Process in Foreground:**
   - [ ] Go to **File → Create Process...**
   - [ ] Enter command: `sleep`
   - [ ] Enter arguments: `5`
   - [ ] Make sure "Run in background" is **unchecked**
   - [ ] Click "Create"
   - [ ] Application should wait ~5 seconds (blocking)
   - [ ] Success message should appear: "Process completed successfully"

2. **Create Process in Background:**
   - [ ] Go to **File → Create Process...**
   - [ ] Enter command: `sleep`
   - [ ] Enter arguments: `60`
   - [ ] **Check** "Run in background"
   - [ ] Click "Create"
   - [ ] Success message should show: "Process created in background with PID: XXXX"
   - [ ] Application should NOT block (returns immediately)
   - [ ] Refresh the process list
   - [ ] Find the process by PID or search for "sleep"
   - [ ] Process should appear in the list

3. **Test Different Commands:**
   ```bash
   # Try creating different processes:
   ```
   - [ ] Command: `echo`, Args: `"Hello World"` (foreground) → should complete quickly
   - [ ] Command: `yes`, Args: `>/dev/null` (background) → should create a high CPU process
   - [ ] Command: `ls`, Args: `-la /tmp` (foreground) → should show output and complete

4. **Test Error Handling:**
   - [ ] Try creating a non-existent command: `nonexistentcommand`
   - [ ] Should show error: "Failed to create process: ..."
   - [ ] Try with empty command → should show "Command cannot be empty"

5. **Verify Created Processes:**
   - [ ] After creating background processes, refresh the list
   - [ ] Created processes should appear with their PIDs
   - [ ] You can select and manage them (kill, pause, etc.)

**Test: Deadlock Detection** 🆕
1. **Understanding Deadlock Detection:**
   - Deadlock detection flags processes in uninterruptible sleep (D state) for >30 seconds
   - This is a heuristic - real deadlocks are complex to detect

2. **Check for Existing Deadlocked Processes:**
   - [ ] Look through the process list
   - [ ] Processes with state 'D' (Disk sleep) that have been running >30 seconds
   - [ ] Should be highlighted in **YELLOW** (abnormal)
   - [ ] Click on such a process
   - [ ] Details panel should show warning: "Possible deadlock (uninterruptible sleep for Xs)"

3. **Create a Test Scenario (if possible):**
   ```bash
   # This is tricky - deadlocks are hard to create intentionally
   # But you can look for processes stuck in D state:
   # Check processes that are waiting on I/O
   ```
   - [ ] Look for processes in D state (usually I/O operations)
   - [ ] If any have been running >30 seconds, they should be flagged
   - [ ] Check the abnormality reason in details panel

4. **Verify Deadlock Warning:**
   - [ ] Find a process with state 'D' and uptime >30s
   - [ ] Should show in details: "⚠️ Warning: Possible deadlock (uninterruptible sleep for Xs)"
   - [ ] Process should be marked as abnormal (yellow highlight)

**Test: Resource Thresholds**
1. Go to **View → Configure Thresholds**
2. Set CPU threshold to 50%
3. Set Memory threshold to 500 MB
4. Close window
5. [ ] Processes exceeding thresholds highlighted in **RED**
6. [ ] CPU column shows red for high CPU processes
7. [ ] Memory column shows red for high memory processes
8. [ ] Create a high CPU process (see above) → should turn red when threshold exceeded

**Test: Color-Coded States**
- [ ] **Green (R)** = Running processes
- [ ] **Blue (S)** = Sleeping processes
- [ ] **Yellow (Z)** = Zombie processes
- [ ] **Red (D)** = Disk sleep
- [ ] **Gray (T)** = Stopped

### 6. Process Tree View ✅

**Test: Tree View Toggle**
1. Go to **View → Process Tree View** (check the box)
2. [ ] View switches to tree format
3. [ ] Shows parent-child relationships
4. [ ] Processes are nested under parents
5. [ ] Can still select processes in tree view
6. Uncheck → returns to table view

**Test: Tree View Interaction**
- [ ] Click on processes in tree → details panel updates
- [ ] Can check boxes for batch selection
- [ ] Tree structure is correct (children under parents)

### 7. Auto-Refresh ✅

**Test: Auto-Refresh**
- [ ] **View → Auto Refresh** is checked by default
- [ ] Status bar shows "Last refresh: X.Xs ago"
- [ ] Process list updates automatically every 2 seconds
- [ ] Uncheck auto-refresh → stops updating
- [ ] Manual refresh button still works

**Test: Manual Refresh**
- [ ] Click **🔄 Refresh** button
- [ ] Process list updates immediately
- [ ] Or use **File → Refresh**

### 8. Error Handling ✅

**Test: Invalid Input**
- [ ] Enter invalid priority (e.g., "abc") → error message
- [ ] Enter priority out of range → may show error
- [ ] Try to kill non-existent PID → appropriate error

**Test: Network/System Errors**
- [ ] If `/proc` is inaccessible → error message
- [ ] If process disappears during operation → graceful handling

### 9. Menu Functions ✅

**Test: File Menu**
- [ ] **File → Refresh** works
- [ ] **File → Exit** closes application

**Test: View Menu**
- [ ] **View → Auto Refresh** toggles
- [ ] **View → Process Tree View** toggles
- [ ] **View → Configure Thresholds** opens window
- [ ] Sort options work

**Test: Operations Menu**
- [ ] All batch operations accessible
- [ ] Operations work when processes are selected

## Safe Testing Practices

### ✅ Safe to Test:
- Viewing processes
- Searching and sorting
- Viewing process details
- Setting priority on your own processes
- Pausing/resuming your own processes
- Killing your own test processes

### ⚠️ Be Careful:
- Killing system processes (can crash system)
- Killing processes you don't own (may need sudo)
- Batch operations on many processes
- Changing priority of system processes

### ❌ Don't Test:
- Killing init (PID 1) - will crash system
- Killing kernel threads
- Killing processes you don't understand

## Test Scenarios

### Scenario 1: Basic Usage
1. Open application
2. Search for "bash"
3. Click on your terminal process
4. View details
5. Sort by memory
6. Enable tree view
7. Disable tree view

### Scenario 2: Process Management
1. Open a test terminal
2. Run a long command (e.g., `sleep 60`)
3. Find the process in the GUI
4. Pause it
5. Resume it
6. Set its priority to 10
7. Terminate it

### Scenario 3: Monitoring
1. Configure thresholds (CPU: 50%, Memory: 500MB)
2. Identify processes exceeding thresholds
3. Look for zombie processes
4. Check process tree for relationships

### Scenario 4: Batch Operations
1. Open 3-4 test terminals
2. Find all their processes
3. Select them all
4. Pause all
5. Resume all
6. Kill all

## Expected Results

### Success Indicators:
- ✅ Operations complete without errors
- ✅ Success messages appear
- ✅ Process list updates correctly
- ✅ No crashes or freezes
- ✅ Error messages are clear and helpful

### Common Issues:
- ❌ "Permission denied" → Need to run with `sudo`
- ❌ Process not found → Process may have exited
- ❌ Operation failed → Check if process exists and you have permissions

## Performance Testing

- [ ] Application starts quickly (< 2 seconds)
- [ ] Process list loads smoothly
- [ ] Sorting is responsive
- [ ] Search filtering is instant
- [ ] Auto-refresh doesn't cause lag
- [ ] Can handle 100+ processes without issues

## Reporting Issues

If you find bugs, note:
1. What you were trying to do
2. What happened vs. what you expected
3. Error messages (if any)
4. Steps to reproduce
5. System information (Ubuntu version, Rust version)

---

**Happy Testing! 🚀**

