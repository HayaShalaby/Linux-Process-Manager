# Linux Process Manager

A modern GUI application for managing and monitoring Linux processes, built with Rust and egui.

## Features

- **Real-time Process Monitoring**: View all running processes with live updates
- **Process Details**: Detailed information including PID, UID, state, memory usage, and priority
- **Search & Filter**: Quickly find processes by name, PID, or UID
- **Sortable Columns**: Sort processes by PID, name, memory, CPU, or other attributes
- **Auto-refresh**: Automatically refresh process list at configurable intervals
- **Process Tree View**: Visualize parent-child process relationships
- **Abnormal Process Detection**: Automatically flags zombie processes and processes exceeding resource thresholds
- **Batch Operations**: Select and operate on multiple processes simultaneously
- **Process Operations**: Kill, terminate, pause, resume, and set priority
- **Modern GUI**: Clean, responsive interface built with egui

## Requirements

- **Operating System**: Linux (Ubuntu recommended)
- **Rust**: Version 1.70 or later
- **Dependencies**: 
  - Access to `/proc` filesystem (standard on Linux)
  - X11 or Wayland display server for GUI

## Installation

### 1. Install Rust

If you don't have Rust installed, use rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install System Dependencies (Ubuntu/Debian)

For GUI support, you may need to install additional libraries:

```bash
sudo apt-get update
sudo apt-get install -y \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev
```

### 3. Clone and Build

```bash
# Clone the repository
git clone https://github.com/HayaShalaby/Linux-Process-Manager.git
cd Linux-Process-Manager

# Build the project
cargo build --release
```

## Running the Application

### Run from source:

```bash
cargo run --release
```

### Run the compiled binary:

```bash
./target/release/lpm_backend
```

## Usage

1. **View Processes**: The main window displays all running processes in a sortable table
2. **Search**: Use the search bar to filter processes by name, PID, or UID
3. **Sort**: Click on column headers (PID, Name, CPU, Memory, etc.) to sort processes
4. **View Details**: Click on any process row to view detailed information in the bottom panel
5. **Process Operations**: Select a process and use the action buttons (Kill, Terminate, Pause, Resume, Set Priority)
6. **Batch Operations**: Check multiple processes and use Operations menu for batch actions
7. **Process Tree**: Toggle tree view from View menu to see parent-child relationships
8. **Configure Thresholds**: Set CPU and memory thresholds to highlight abnormal processes
9. **Refresh**: Click the "🔄 Refresh" button or use File → Refresh to update the process list
10. **Auto-refresh**: Enable/disable auto-refresh from the View menu

## Process States

Process states are displayed as single letters and color-coded in the GUI:

- **R (Running)** 🟢 - Process is currently executing or ready to run
- **S (Sleeping)** 🔵 - Process is waiting for an event (interruptible sleep)
- **D (Disk Sleep)** 🔴 - Process is in uninterruptible sleep (usually waiting for I/O)
- **Z (Zombie)** 🟡 - Process has terminated but parent hasn't reaped it yet
- **T (Stopped)** ⚪ - Process has been stopped (by a signal or debugger)
- **I (Idle)** - Kernel thread in idle state (rarely seen)
- **W (Waking)** - Process is waking up (transitional state)
- **K (Wakekill)** - Process is marked for wake kill
- **X (Dead)** - Process is dead (should never appear)

**Note**: Zombie processes (Z) are automatically highlighted in yellow as abnormal processes. They are harmless and will be cleaned up automatically by the system.

## Project Structure

```
src/
├── main.rs          # Application entry point (GUI launcher)
├── user.rs          # User and privilege system
├── manager.rs       # Manager struct and process management
├── process/         # Process data structures and parsing
│   ├── mod.rs       # Process struct and TryFrom implementation
│   ├── pcb.rs       # Process Control Block data (CPU, memory, state, priority)
│   ├── tree.rs      # Process tree structure for parent-child relationships
│   └── scheduler.rs # Placeholder for scheduler module
├── manager/         # Process management operations (Ismail's backend)
│   ├── operations.rs    # Process operations: kill, terminate, pause, resume, set_priority
│   ├── batch.rs         # Batch operations and process tree building
│   ├── monitoring.rs    # Process monitoring and refresh functionality
│   └── permissions.rs   # Permission checking (Admin required)
└── gui/             # GUI application (TLI - Refai's implementation)
    ├── mod.rs       # GUI module exports
    └── app.rs       # Main GUI application logic with all features
```

## Development

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

## Known Limitations & Missing Features

### Currently Not Implemented

1. **Process Timer/Uptime**
   - Process runtime/uptime is not displayed
   - Would require tracking process start time from `/proc/[pid]/stat`

### Fully Implemented Features

✅ Process listing and display  
✅ Process search and filtering  
✅ Sorting by all columns (PID, Name, UID, State, CPU, Memory, Priority)  
✅ Process tree view  
✅ **CPU percentage calculation** - Real-time CPU usage tracking with proper jiffies-to-percentage conversion  
✅ Memory usage calculation and display  
✅ Process state detection and color coding  
✅ Zombie process detection  
✅ CPU and memory threshold monitoring with visual indicators  
✅ Memory threshold monitoring  
✅ Process operations (kill, terminate, pause, resume, set priority)  
✅ Batch operations  
✅ Permission system (Admin required for operations)  
✅ Auto-refresh functionality  

## Troubleshooting

### GUI doesn't open

- Ensure you have a display server running (X11 or Wayland)
- Check that required system libraries are installed (see Installation section)
- Try running with `DISPLAY=:0 cargo run` if using X11

### Permission errors

- Most operations (kill, pause, resume, set priority) require **Admin privileges**
- Run with `sudo` to perform operations: `sudo ./target/release/lpm_backend`
- Reading process list works without sudo, but operations will fail with "Permission denied"

### CPU shows 0.0%

- This is expected - CPU calculation is not yet implemented
- CPU threshold monitoring is ready but waiting for backend implementation
- Memory usage and other metrics work correctly

### Build errors

- Ensure Rust is up to date: `rustup update`
- Clean and rebuild: `cargo clean && cargo build --release`

## Contributors

- Backend: Ismail & Haya
- GUI (TLI): Refai & Hana
