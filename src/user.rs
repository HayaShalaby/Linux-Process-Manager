
// Defines the privilege level of the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Privilege {
    Normal,
    Admin,
}

//Represents the active user interacting with the Linux Process Manager.
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub privilege: Privilege,
}

impl User {
    //Creates a basic User instance for initial manager setup.
    pub fn new(id: u32, name: &str, privilege: Privilege) -> Self {
        User {
            id,
            name: name.to_string(),
            privilege,
        }
    }

    //Placeholder method to check if the user is an admin.
    pub fn is_admin(&self) -> bool {
        self.privilege == Privilege::Admin
    }
}