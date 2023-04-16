mod memory;

#[derive(Debug)]
pub struct User {
    email: String,
    password_hash: u64,
}

pub trait Storage {
    type Error;

    fn add_user(&mut self, user: User) -> Result<(), Self::Error>;
    fn remove_user(&mut self, email: &str) -> Result<User, Self::Error>;
}
