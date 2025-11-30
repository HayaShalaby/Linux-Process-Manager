
use procfs::{
    process::Process as ProcfsProcess,
    ProcError,
};
use std::convert::TryFrom;

// 1. Declare submodules
mod pcb; 
mod scheduler; // Placeholder
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
}
