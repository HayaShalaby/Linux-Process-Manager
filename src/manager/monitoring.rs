use std::collections::HashMap;
use std::convert::TryFrom;
use procfs;

use crate::process::Process;

// Reads the /proc filesystem, updates the provided HashMap with current data, and returns the number of processes successfully loaded.
pub fn refresh_processes(processes: &mut HashMap<u32, Process>) -> Result<usize, String> {
    
    let procfs_processes = match procfs::process::all_processes() { //Reading intial process list
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to read process list: {}", e)),
    };

    let mut new_processes = HashMap::new(); //New temporary hash_map to store the new process list 
    let mut successfully_loaded = 0;
    
    //Loop over new process info, validate it, and add it to the new hash_map
    for p in procfs_processes { //Loops every process that procfs managed to find
        let procfs_proc = match p {
            Ok(p) => p,
            Err(_) => continue, //Skip listing errors
        };

        let pid = procfs_proc.pid as u32;

        match Process::try_from(pid) {
            Ok(proc) => {
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

    
    *processes = new_processes; //Replace the old process map with the new one
    
    Ok(successfully_loaded)
}
