use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub verified: bool,
}

impl User {
    pub fn default() -> Self {
        User {
            id: Uuid::nil(),
            username: String::from(""),
            email: String::from(""),
            verified: false,
        }
    }
}
