use std::process::{Command, Stdio};
use crate::manager::Manager;
use crate::manager::permissions;

/// Create a new process in foreground mode (blocking)
/// The process will run and block until it completes
pub fn create_process_foreground(manager: &Manager, command: &str, args: &[&str]) -> Result<i32, String> {
    permissions::check_admin_privilege(manager)?;
    
    let mut cmd = Command::new(command);
    cmd.args(args);
    
    // In foreground mode, we wait for the process to complete
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                Ok(status.code().unwrap_or(0))
            } else {
                Err(format!("Process exited with code: {}", status.code().unwrap_or(-1)))
            }
        }
        Err(e) => Err(format!("Failed to execute process: {}", e))
    }
}

/// Create a new process in background mode (non-blocking)
/// Returns the PID of the spawned process
pub fn create_process_background(manager: &Manager, command: &str, args: &[&str]) -> Result<u32, String> {
    permissions::check_admin_privilege(manager)?;
    
    let mut cmd = Command::new(command);
    cmd.args(args);
    
    // Redirect stdin, stdout, stderr to /dev/null for background process
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    
    // Spawn the process in the background
    // Note: child.id() returns the PID on Unix systems
    match cmd.spawn() {
        Ok(child) => {
            // On Unix, id() returns the PID
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                Ok(child.id() as u32)
            }
            #[cfg(not(unix))]
            {
                Err("Background process creation only supported on Unix systems".to_string())
            }
        }
        Err(e) => Err(format!("Failed to spawn background process: {}", e))
    }
}

/// Create a process with shell execution (supports shell features like pipes, redirects)
pub fn create_process_shell(manager: &Manager, shell_command: &str, background: bool) -> Result<u32, String> {
    permissions::check_admin_privilege(manager)?;
    
    if background {
        // Background: spawn and return PID
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(shell_command);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        
        match cmd.spawn() {
            Ok(child) => Ok(child.id() as u32),
            Err(e) => Err(format!("Failed to spawn background shell process: {}", e))
        }
    } else {
        // Foreground: execute and wait
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(shell_command);
        
        match cmd.status() {
            Ok(status) => {
                if status.success() {
                    Ok(0) // Success exit code
                } else {
                    Err(format!("Process exited with code: {}", status.code().unwrap_or(-1)))
                }
            }
            Err(e) => Err(format!("Failed to execute shell process: {}", e))
        }
    }
}

