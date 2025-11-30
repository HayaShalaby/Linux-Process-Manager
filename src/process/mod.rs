
use procfs::{
    process::Process as ProcfsProcess,
    ProcError,
};
use std::convert::TryFrom;

// 1. Declare submodules
mod pcb; 
pub mod tree;

// 2. Import the public PcbData struct from the pcb submodule
use pcb::PcbData; 


// Main Process Data Structure 

/// Represents a single process on the system.
#[derive(Debug, Clone)]
pub struct Process {
    pub process_id: u32,
    pub user_id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub pcb_data: PcbData, 
}


// Implementation

impl TryFrom<u32> for Process {
    type Error = ProcError;

    fn try_from(pid: u32) -> Result<Self, Self::Error> {
        let procfs_proc = ProcfsProcess::new(pid as i32)?;
        let stat = procfs_proc.stat()?;
        let status = procfs_proc.status()?;
        let page_size: u64 = procfs::page_size();
        let memory_rss_mb = (stat.rss as u64 * page_size) / 1024 / 1024;
        let user_id = status.ruid;
        let cpu_percent_placeholder = 0.0;
        
        // Calculate process uptime/runtime
        // starttime is in jiffies since system boot
        // We need to get system uptime and calculate the difference
        let uptime_seconds = Self::calculate_uptime(stat.starttime as u64)?;

        // 3. Construct the custom Process struct
        Ok(Process {
            process_id: pid,
            user_id,
            name: stat.comm,
            parent_id: Some(stat.ppid as u32), 
            pcb_data: PcbData { 
                cpu_percent: cpu_percent_placeholder,
                memory_rss_mb,
                state: stat.state,
                priority: stat.nice as i32,
                uptime_seconds,
            },
        })
    }
}

impl Process {
    /// Update the CPU percentage for this process
    pub fn set_cpu_percent(&mut self, cpu_percent: f32) {
        self.pcb_data.cpu_percent = cpu_percent;
    }
    
    /// Get the total CPU time (utime + stime) in jiffies from /proc/[pid]/stat
    pub fn get_cpu_time_jiffies(pid: u32) -> Result<u64, ProcError> {
        let procfs_proc = ProcfsProcess::new(pid as i32)?;
        let stat = procfs_proc.stat()?;
        // Total CPU time = user time + system time (in jiffies)
        Ok(stat.utime as u64 + stat.stime as u64)
    }
    
    /// Calculate process uptime in seconds
    /// starttime is in jiffies since system boot
    fn calculate_uptime(starttime_jiffies: u64) -> Result<u64, ProcError> {
        // Get system uptime from /proc/uptime
        let uptime_str = std::fs::read_to_string("/proc/uptime")
            .map_err(|_| ProcError::NotFound(None))?;
        let system_uptime_secs: f64 = uptime_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| ProcError::NotFound(None))?;
        
        // Get system HZ (clock ticks per second)
        let hz = Self::get_system_hz();
        let system_uptime_jiffies = (system_uptime_secs * hz) as u64;
        
        // Process uptime = system uptime - process start time
        if system_uptime_jiffies > starttime_jiffies {
            Ok((system_uptime_jiffies - starttime_jiffies) / hz as u64)
        } else {
            Ok(0) // Process started before system boot (shouldn't happen, but handle gracefully)
        }
    }
    
    /// Get system HZ (clock ticks per second)
    fn get_system_hz() -> f64 {
        unsafe extern "C" {
            fn sysconf(name: i32) -> i64;
        }
        
        unsafe {
            // _SC_CLK_TCK = 2
            let hz = sysconf(2);
            if hz > 0 {
                return hz as f64;
            }
        }
        
        // Fallback to 100 (standard Linux HZ)
        100.0
    }
    
    /// Format uptime as human-readable string (e.g., "1h 23m 45s" or "5m 30s")
    pub fn format_uptime(&self) -> String {
        let seconds = self.pcb_data.uptime_seconds;
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}
