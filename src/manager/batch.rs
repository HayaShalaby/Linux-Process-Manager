use std::collections::HashMap;
use crate::manager::Manager;
use crate::manager::operations;
use crate::process::{Process};
use crate::process::tree::ProcessNode;

//Building the process_tree which will be used heavily especially with parent-child relationships
pub fn build_process_tree(manager: &Manager) -> Option<ProcessNode> {
    
    let root_process = match manager.processes.get(&manager.root_pid) {
        Some(p) => p.clone(), 
        None => return None,
    };
    
    let mut root_node = ProcessNode::new(root_process);
    
    //Map: Parent PID -> List of direct child Process structs
    let mut children_map: HashMap<u32, Vec<Process>> = HashMap::new();
    

    for (&pid, process) in manager.processes.iter() {
    if pid == manager.root_pid {
        continue;
    }

    if let Some(ppid) = process.parent_id {
        children_map
            .entry(ppid)
            .or_insert_with(Vec::new)
            .push(process.clone());
    }
}

    
    //Recursive helper function to build the tree from the top down
    fn build_node(node: &mut ProcessNode, children_map: &HashMap<u32, Vec<Process>>) {
        let pid = node.process.process_id;
        
        if let Some(children) = children_map.get(&pid) {
            for child_process in children {
                let mut child_node = ProcessNode::new(child_process.clone());
                //Recursively build children of this child
                build_node(&mut child_node, children_map);
                node.children.push(child_node);
            }
        }
    }

    build_node(&mut root_node, &children_map);
    
    Some(root_node)
}

fn get_descendant_pids(node: &ProcessNode) -> Vec<u32> {
    let mut descendants = Vec::new();
    
    for child in &node.children {
        //Add the child's PID
        descendants.push(child.process.process_id);
        
        //Recursively find and add the child's descendants (grandchildren, etc.)
        descendants.extend(get_descendant_pids(child));
    }
    
    descendants
}

//Placeholder for a group action, like killing a process and all its children.
pub fn kill_descendants(manager: &Manager, parent_pid: u32) -> Result<Vec<u32>, String> {
    // 1. Permission Check: Batch actions require Admin privileges.
    crate::manager::permissions::check_admin_privilege(manager)?;


    //Build the entire process tree structure
    let root_node = manager.build_process_tree()
        .ok_or_else(|| "Failed to build process tree.".to_string())?;

    //Find the starting node (the parent to be killed) in the tree
    let mut stack = vec![&root_node];
    let mut parent_node: Option<&ProcessNode> = None;

    //Use a simple iterative search to find the parent node
    while let Some(current) = stack.pop() {
        if current.process.process_id == parent_pid {
            parent_node = Some(current);
            break;
        }
        stack.extend(current.children.iter());
    }

    let start_node = parent_node.ok_or_else(|| format!("PID {} not found in active processes.", parent_pid))?;

    //Get all descendant PIDs using the recursive helper
    let pids_to_kill = get_descendant_pids(start_node);
    
    //Execute the kill operation on all descendants
    let mut successful_kills = Vec::new();
    let mut failed_kills = 0;
    
    for pid in pids_to_kill.iter().rev() {
        //Kill children first to prevent accidental reparenting
        match operations::kill_process(manager, *pid) {
            Ok(_) => successful_kills.push(*pid),
            Err(e) => {
                eprintln!("Warning: Failed to kill descendant PID {}: {}", pid, e);
                failed_kills += 1;
            }
        }
    }

    //Optionally kill the parent last, if required by the user's intent.
    // For this implementation, we will only return the descendants killed.
    
    if failed_kills > 0 {
        return Err(format!("Successfully killed {} processes, but failed to kill {} descendants.", successful_kills.len(), failed_kills));
    }

    Ok(successful_kills)
}   
