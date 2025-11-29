
mod process; 
mod user;   
mod manager;


use std::convert::TryFrom;
use process::Process; 
use procfs::ProcError; // Need ProcError to match the error type

fn main() {
    println!("inux Process Manager: Process Extraction Demo");
    
    let mut processes: Vec<Process> = Vec::new();
    
    // 1. Get the list of all PIDs, and collect the iterator into a Vec to get the count
    match procfs::process::all_processes() {
        Ok(procfs_processes_iter) => {
            let procfs_processes: Vec<_> = procfs_processes_iter.collect();
            
            println!("Attempting to read {} processes...", procfs_processes.len());
            
            // 2. Iterate and convert
            for procfs_proc_result in procfs_processes {
                
                let procfs_proc = match procfs_proc_result {
                    Ok(p) => p,
                    Err(_) => continue, // Skip any errors when listing PIDs
                };
                
                // Get the PID
                let pid = procfs_proc.pid as u32;

                // Use the TryFrom trait implemented in src/process/mod.rs
                match Process::try_from(pid) {
                    Ok(p) => processes.push(p),
                    Err(ProcError::NotFound(_)) => {
                        // Process may have exited after the list was created, ignore
                    },
                    Err(_) => {
                        // Ignore other errors during read
                    }
                }
            }

            // 3. Print the results
            println!("\n Successfully loaded {} active processes.", processes.len());
            println!("---------------------------------------------------------------------------------");
            println!("{: <5} | {: <5} | {: <10} | {: <10} | Name", "PID", "UID", "STATE", "RSS (MB)");
            println!("---------------------------------------------------------------------------------");

            for p in processes.iter() {
                println!(
                    "{: <5} | {: <5} | {: <10} | {: <10} | {}",
                    p.process_id,
                    p.user_id,
                    p.pcb_data.state,
                    p.pcb_data.memory_rss_mb,
                    p.name
                );
            }
            println!("---------------------------------------------------------------------------------");
        }
        Err(e) => {
            eprintln!(" Failed to read /proc filesystem: {:?}", e);
        }
    }
}
