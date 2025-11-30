use crate::process::Process; //Import the base Process struct

//Represents a node in the process tree hierarchy.
//The hierarchy is built by attaching children nodes to their parent.
pub struct ProcessNode {
    pub process: Process, 
    pub children: Vec<ProcessNode>,
}

impl ProcessNode {
    pub fn new(process: Process) -> Self {
        ProcessNode {
            process,
            children: Vec::new(),
        }
    }
}