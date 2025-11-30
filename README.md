# Linux Process Manager

A modern GUI application for managing and monitoring Linux processes, built with Rust and egui.

## Features

- **Real-time Process Monitoring**: View all running processes with live updates
- **Process Details**: Detailed information including PID, UID, state, memory usage, and priority
- **Search & Filter**: Quickly find processes by name, PID, or UID
- **Sortable Columns**: Sort processes by PID, name, memory, or other attributes
- **Auto-refresh**: Automatically refresh process list at configurable intervals
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

# Switch to the RefaiGUI branch (or TLI-GUI branch)
git checkout RefaiGUI

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
3. **Sort**: Click on column headers (Name, Memory) to sort processes
4. **View Details**: Click on any process row to view detailed information in the bottom panel
5. **Refresh**: Click the "🔄 Refresh" button or use File → Refresh to update the process list
6. **Auto-refresh**: Enable/disable auto-refresh from the View menu

## Project Structure

```
src/
├── main.rs          # Application entry point
├── process/         # Process data structures and parsing
│   ├── mod.rs
│   ├── pcb.rs       # Process Control Block data
│   └── tree.rs      # Process tree structure
├── manager/         # Process management operations
│   ├── mod.rs
│   ├── operations.rs
│   ├── monitoring.rs
│   └── permissions.rs
├── gui/             # GUI application (TLI)
│   ├── mod.rs
│   └── app.rs       # Main GUI application logic
└── kernel/          # Kernel interface modules
```

## Branches

- `main`: Main development branch
- `backend--Ismaiel`: Backend implementation by Ismail
- `RefaiGUI`: Complete GUI implementation with all checklist features (Refai)
- `TLI-GUI`: GUI (Text/Linux Interface) implementation

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

## Troubleshooting

### GUI doesn't open

- Ensure you have a display server running (X11 or Wayland)
- Check that required system libraries are installed (see Installation section)
- Try running with `DISPLAY=:0 cargo run` if using X11

### Permission errors

- The application reads from `/proc` which requires appropriate permissions
- Some process details may not be accessible without root privileges

### Build errors

- Ensure Rust is up to date: `rustup update`
- Clean and rebuild: `cargo clean && cargo build --release`

## License

See LICENSE file for details.

## Contributors

- Backend: Ismail & Haya
- GUI (TLI): Refai & Hana