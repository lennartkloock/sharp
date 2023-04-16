use crate::storage::{Storage, User};
use std::collections::HashMap;

pub struct MemoryStorage {
    users: HashMap<String, User>,
}

impl Storage for MemoryStorage {
    type Error = ();

    fn add_user(&mut self, user: User) -> Result<(), Self::Error> {
        self.users.insert(user.email.clone(), user);
        Ok(())
    }

    fn remove_user(&mut self, email: &str) -> Result<User, Self::Error> {
        self.users.remove(email).ok_or(())
    }
}
