use crate::process::Process; // Import the base Process struct

/// Represents a node in the process tree hierarchy.
/// The hierarchy is built by attaching children nodes to their parent.
pub struct ProcessNode {
    /// The actual process data
    pub process: Process, 
    /// List of child nodes belonging to this process
    pub children: Vec<ProcessNode>,
}

impl ProcessNode {
    // Constructor method
    pub fn new(process: Process) -> Self {
        ProcessNode {
            process,
            children: Vec::new(),
        }
    }
}