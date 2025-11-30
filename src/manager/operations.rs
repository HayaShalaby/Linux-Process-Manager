use nix::sys::signal::{self, Signal};
use nix::sys::resource::{self, UsageWho};
use nix::unistd::Pid;

use libc::{setpriority, PRIO_PROCESS};
use crate::manager::permissions;
use crate::manager::Manager;


//Kill (Force terminate)
pub fn kill_process(manager: &Manager, pid: u32) -> Result<(), String> {
    permissions::check_admin_privilege(manager)?;

    let nix_pid = Pid::from_raw(pid as i32);

    signal::kill(nix_pid, Signal::SIGKILL)
        .map_err(|e| format!("Failed to send SIGKILL to PID {}: {}", pid, e))
}

//Terminate (Graceful stop)
//Sends SIGTERM, giving process a chance to shut down cleanly
pub fn terminate_process(manager: &Manager, pid: u32) -> Result<(), String> {
    permissions::check_admin_privilege(manager)?;

    let nix_pid = Pid::from_raw(pid as i32);

    signal::kill(nix_pid, Signal::SIGTERM)
        .map_err(|e| format!("Failed to send SIGTERM to PID {}: {}", pid, e))
}


//Pause (SIGSTOP)
//Fully pauses a process without killing it
pub fn pause_process(manager: &Manager, pid: u32) -> Result<(), String> {
    permissions::check_admin_privilege(manager)?;

    let nix_pid = Pid::from_raw(pid as i32);

    signal::kill(nix_pid, Signal::SIGSTOP)
        .map_err(|e| format!("Failed to pause PID {}: {}", pid, e))
}


//Resume (SIGCONT)
//Resumes a paused process
pub fn resume_process(manager: &Manager, pid: u32) -> Result<(), String> {
    permissions::check_admin_privilege(manager)?;

    let nix_pid = Pid::from_raw(pid as i32);

    signal::kill(nix_pid, Signal::SIGCONT)
        .map_err(|e| format!("Failed to resume PID {}: {}", pid, e))
}


//Set Priority (nice value)
pub fn set_priority(manager: &Manager, pid: u32, nice_value: i32) -> Result<(), String> {
    permissions::check_admin_privilege(manager)?;

    let res = unsafe {
        setpriority(PRIO_PROCESS, pid as u32, nice_value)
    };

    if res == 0 {
        Ok(())
    } else {
        Err(format!(
            "Failed to set nice value for PID {}: {}",
            pid,
            std::io::Error::last_os_error()
        ))
    }
}
