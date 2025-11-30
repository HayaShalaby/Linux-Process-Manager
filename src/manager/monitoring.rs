use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::Instant;
use procfs;

use crate::process::Process;

/// Get the number of CPU cores for CPU percentage calculation
fn get_num_cores() -> f32 {
    // Try to read from /proc/cpuinfo or use sysconf
    // For simplicity, we'll use a fallback method
    match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(content) => {
            content.lines()
                .filter(|line| line.starts_with("processor"))
                .count() as f32
        }
        Err(_) => {
            // Fallback: try to get from sysconf or default to 1
            // On Linux, we can also check /proc/stat
            match std::fs::read_to_string("/proc/stat") {
                Ok(stat_content) => {
                    stat_content.lines()
                        .filter(|line| line.starts_with("cpu") && !line.starts_with("cpu "))
                        .count() as f32
                }
                Err(_) => 1.0, // Default to 1 core if we can't determine
            }
        }
    }
}

/// Get the system HZ (clock ticks per second) for CPU time calculation
/// This is typically 100 on most Linux systems, but can be 1000 on newer kernels
fn get_hz() -> f64 {
    // Try to get from sysconf first (more reliable)
    unsafe {
        extern "C" {
            fn sysconf(name: i32) -> i64;
        }
        // _SC_CLK_TCK = 2
        let hz = sysconf(2);
        if hz > 0 {
            return hz as f64;
        }
    }
    
    // Fallback: read from /proc/self/stat or use default
    // Most modern Linux systems use 100
    match std::fs::read_to_string("/proc/self/stat") {
        Ok(_) => {
            // If we can read /proc, assume standard HZ=100
            // We could parse it more precisely, but 100 is the safe default
            100.0
        }
        Err(_) => 100.0, // Default fallback
    }
}

// Reads the /proc filesystem, updates the provided HashMap with current data, and returns the number of processes successfully loaded.
// Also calculates CPU percentage by tracking CPU time between refreshes.
pub fn refresh_processes(
    processes: &mut HashMap<u32, Process>,
    previous_cpu_times: &mut HashMap<u32, (u64, Instant)>,
) -> Result<usize, String> {
    
    let procfs_processes = match procfs::process::all_processes() { //Reading intial process list
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to read process list: {}", e)),
    };

    let mut new_processes = HashMap::new(); //New temporary hash_map to store the new process list 
    let mut successfully_loaded = 0;
    let current_time = Instant::now();
    let num_cores = get_num_cores();
    let hz = get_hz();
    
    //Loop over new process info, validate it, and add it to the new hash_map
    for p in procfs_processes { //Loops every process that procfs managed to find
        let procfs_proc = match p {
            Ok(p) => p,
            Err(_) => continue, //Skip listing errors
        };

        let pid = procfs_proc.pid as u32;

        match Process::try_from(pid) {
            Ok(mut proc) => {
                // Calculate CPU percentage if we have previous data
                if let Some((prev_cpu_time, prev_time)) = previous_cpu_times.get(&pid) {
                    // Get current CPU time
                    match Process::get_cpu_time_jiffies(pid) {
                        Ok(current_cpu_time) => {
                            let delta_cpu_time = current_cpu_time.saturating_sub(*prev_cpu_time);
                            let delta_wall_time = current_time.duration_since(*prev_time).as_secs_f64();
                            
                            // Calculate CPU percentage
                            // CPU% = (delta_cpu_time / delta_wall_time) * 100 / num_cores
                            // Convert jiffies to seconds using system HZ
                            let cpu_time_seconds = delta_cpu_time as f64 / hz;
                            
                            if delta_wall_time > 0.0 {
                                let cpu_percent = (cpu_time_seconds / delta_wall_time) * 100.0 / num_cores;
                                proc.set_cpu_percent(cpu_percent as f32);
                            } else {
                                proc.set_cpu_percent(0.0);
                            }
                            
                            // Update previous CPU time
                            previous_cpu_times.insert(pid, (current_cpu_time, current_time));
                        }
                        Err(_) => {
                            // If we can't get CPU time, keep previous value or set to 0
                            proc.set_cpu_percent(0.0);
                        }
                    }
                } else {
                    // First time seeing this process - no CPU percentage yet
                    // Store current CPU time for next refresh
                    if let Ok(cpu_time) = Process::get_cpu_time_jiffies(pid) {
                        previous_cpu_times.insert(pid, (cpu_time, current_time));
                    }
                    proc.set_cpu_percent(0.0);
                }
                
                new_processes.insert(pid, proc);
                successfully_loaded += 1;
            }
            Err(e) => {
                //Ignore the error if a process vanished between listing and reading its data
                if !matches!(e, procfs::ProcError::NotFound(_)) {
                    eprintln!("Warning: Could not fully read data for PID {}: {:?}", pid, e);
                }
            }
        }
    }

    // Clean up old entries from previous_cpu_times for processes that no longer exist
    previous_cpu_times.retain(|pid, _| new_processes.contains_key(pid));
    
    *processes = new_processes; //Replace the old process map with the new one
    
    Ok(successfully_loaded)
}
