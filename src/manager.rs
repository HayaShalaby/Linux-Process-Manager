use std::collections::HashMap;
use std::time::Instant;
use crate::process::Process;
use crate::user::User;

pub mod batch;
pub mod monitoring;
pub mod operations;
pub mod permissions;
pub mod creation;


#[derive(Debug)] //Allows an instance of the Manager struct to be formatted for debugging output in a human-readable way.

//Manager struct declaration
pub struct Manager {
    pub processes: HashMap<u32, Process>, 
    pub active_user: User, 
    pub root_pid: u32,
    // Track previous CPU times for CPU percentage calculation
    // HashMap<pid, (cpu_time_jiffies, timestamp)>
    pub(crate) previous_cpu_times: HashMap<u32, (u64, Instant)>,
}

impl Manager {
    pub fn new(active_user: User) -> Result<Self, String> {  //Constructor
        //Initialize the struct with default state
        let mut manager = Manager {
            processes: HashMap::new(), // Start with an empty map
            active_user,
            root_pid: 1,
            previous_cpu_times: HashMap::new(),
        };
        
        //Initial snapshot at initialization
        match monitoring::refresh_processes(&mut manager.processes, &mut manager.previous_cpu_times) {
            Ok(_) => Ok(manager),
            Err(e) => Err(format!("Failed initial process load: {}", e)),
        }
    }

    //Deals with live data from Linux system
   pub fn refresh(&mut self) -> Result<(), String> {
    monitoring::refresh_processes(&mut self.processes, &mut self.previous_cpu_times).map(|_| ())
}

    pub fn build_process_tree(&self) -> Option<crate::process::tree::ProcessNode> {
        batch::build_process_tree(self)
    }
 
    pub fn processes(&self) -> Vec<&Process> { //Process getter
        self.processes.values().collect() // Collects references to the Process structs from the HashMap values
    }
}

