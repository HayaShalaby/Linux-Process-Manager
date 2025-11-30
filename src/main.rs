mod process;
mod user;
mod manager;


use std::error::Error;
use std::io::{self, Write};
use std::time::Duration;
use std::{thread};


use crate::manager::Manager;
use crate::manager::operations;
use crate::user::{User, Privilege};

//Reads an interactive command from user

fn read_command() -> String {
    print!("Command > ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}



fn main() -> Result<(), Box<dyn Error>> {
    println!("Linux Process Manager: Process Extraction Demo");

    //Create active user (admin for now)
    let active_user = User::new(
        1000,
        "current_user",
        Privilege::Admin
    );

    println!(
        "User: {} (UID: {} | Privilege: {:?})",
        active_user.name, active_user.id, active_user.privilege
    );

    //Initialize manager
    println!("\nInitializing process manager and loading processes...");
    let mut manager = match Manager::new(active_user) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Fatal Error: Failed to initialize Manager: {}", e);
            return Ok(());
        }
    };

    //Live update + interactive commands
    loop {
        //Refresh list
        if let Err(e) = manager.refresh() {
            eprintln!("Warning: Could not refresh processes: {}", e);
        }

        let processes = manager.processes();
        let count = processes.len();

        //Clear terminal
        print!("\x1B[2J\x1B[1;1H");

        println!("===============================================");
        println!(" ACTIVE PROCESSES ({} total)", count);
        println!("===============================================");
        println!("{: <6} {: <6} {: <6} {: <10} {}", "PID", "UID", "STATE", "RSS(MB)", "NAME");
        println!("------------------------------------------------");

        for p in processes.iter() {
            println!(
                "{: <6} {: <6} {: <6} {: <10} {}",
                p.process_id,
                p.user_id,
                p.pcb_data.state,
                p.pcb_data.memory_rss_mb,
                p.name
            );
        }

        println!("------------------------------------------------");
        println!("Commands:");
        println!(" kill <pid>     | force kill");
        println!(" term <pid>     | graceful terminate");
        println!(" pause <pid>    | SIGSTOP");
        println!(" resume <pid>   | SIGCONT");
        println!(" nice <pid> <value> | set priority");
        println!(" refresh        | refresh now");
        println!(" exit           | quit program");
        println!("------------------------------------------------");

        // Read command
        let cmd = read_command();
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "kill" if parts.len() == 2 => {
                let pid = parts[1].parse().unwrap_or(0);
                match operations::kill_process(&manager, pid) {
                    Ok(_) => println!("Killed process {}", pid),
                    Err(e) => println!("Error: {}", e),
                }
            }

            "term" if parts.len() == 2 => {
                let pid = parts[1].parse().unwrap_or(0);
                match operations::terminate_process(&manager, pid) {
                    Ok(_) => println!("Terminated process {}", pid),
                    Err(e) => println!("Error: {}", e),
                }
            }

            "pause" if parts.len() == 2 => {
                let pid = parts[1].parse().unwrap_or(0);
                match operations::pause_process(&manager, pid) {
                    Ok(_) => println!("Paused process {}", pid),
                    Err(e) => println!("Error: {}", e),
                }
            }

            "resume" if parts.len() == 2 => {
                let pid = parts[1].parse().unwrap_or(0);
                match operations::resume_process(&manager, pid) {
                    Ok(_) => println!("Resumed process {}", pid),
                    Err(e) => println!("Error: {}", e),
                }
            }

            "nice" if parts.len() == 3 => {
                let pid = parts[1].parse().unwrap_or(0);
                let nice: i32 = parts[2].parse().unwrap_or(0);
                match operations::set_priority(&manager, pid, nice) {
                    Ok(_) => println!("Set nice for PID {} to {}", pid, nice),
                    Err(e) => println!("Error: {}", e),
                }
            }

            "refresh" => {
                println!("Manual refresh requested.");
            }

            "exit" => {
                println!("Exiting...");
                break;
            }

            _ => {
                println!("Unknown or invalid command.");
            }
        }

        thread::sleep(Duration::from_secs(2));
    }

    Ok(())
}
