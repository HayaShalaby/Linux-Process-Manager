use crate::manager::Manager;
use crate::user::Privilege;

//Checks if the active user has Admin privileges (Done before execution)
pub fn check_admin_privilege(manager: &Manager) -> Result<(), String> {
    if manager.active_user.privilege == Privilege::Admin {
        Ok(())
    } else {
        Err("Permission denied: Admin privileges required to perform this action.".to_string())
    }
}