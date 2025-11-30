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
/// Uses shell with proper argument escaping to safely detach the process
pub fn create_process_background(manager: &Manager, command: &str, args: &[&str]) -> Result<u32, String> {
    permissions::check_admin_privilege(manager)?;
    
    // Use shell to properly detach the process using double-fork technique
    // This prevents the process from becoming a zombie
    // We properly escape arguments to prevent shell injection
    let mut cmd = Command::new("sh");
    cmd.arg("-c");
    
    // Build the command with properly escaped arguments
    // Using printf %q to safely quote arguments (if available) or manual escaping
    let mut escaped_args = Vec::new();
    for arg in args {
        // Simple escaping: wrap in single quotes and escape single quotes within
        let escaped = arg.replace('\'', "'\"'\"'");
        escaped_args.push(format!("'{}'", escaped));
    }
    
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, escaped_args.join(" "))
    };
    
    // Use nohup and & to properly background the process
    // The shell will handle the double-fork and detach it from our process
    // echo $! outputs the PID of the backgrounded process
    cmd.arg(&format!("nohup {} > /dev/null 2>&1 & echo $!", full_command));
    
    // Redirect stdin to null
    cmd.stdin(Stdio::null());
    
    // Capture the output to get the PID
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                // Parse the PID from stdout
                let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                match pid_str.parse::<u32>() {
                    Ok(pid) => Ok(pid),
                    Err(_) => Err(format!("Failed to parse PID from output: {}", pid_str))
                }
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to create background process: {}", error_msg))
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

