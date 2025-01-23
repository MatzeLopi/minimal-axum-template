use crate::{
    http::{error::Error as HTTPError, utils::random_string},
    schemas::users::User,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_user_by_id(id: &Uuid, db: &PgPool) -> Result<User, HTTPError> {
    /// Get a user by their id
    ///
    /// # Arguments
    ///  id: &Uuid - The user id
    ///  db: PgPool - The database connection pool
    ///
    /// # Returns
    ///  Result<User, HTTPError> - The user if found, an error otherwise
    let result = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
        .fetch_one(db)
        .await;

    match result {
        Ok(row) => Ok(User {
            id: row.id,
            username: row.username,
            email: row.email,
            verified: row.is_verified,
        }),
        Err(e) => Err(HTTPError::from(e)),
    }
}

pub async fn check_username(username: &str, db: &PgPool) -> bool {
    /// Check if the username exists in the DB
    ///
    /// # Arguments
    ///  username: &str - The username to check
    ///  db: PgPool - The database connection pool
    ///
    /// # Returns
    ///  bool - True if the username exists, false otherwise
    let result = sqlx::query!("SELECT username FROM users WHERE username = $1", username)
        .fetch_one(db)
        .await;

    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn check_email(email: &str, db: &PgPool) -> bool {
    /// Check if the email exists in the DB
    ///
    /// # Arguments
    ///  email: &str - The email to check
    ///  db: PgPool - The database connection pool
    ///
    /// # Returns
    ///  bool - True if the email exists, false otherwise
    let result = sqlx::query!("SELECT email FROM users WHERE email = $1", email)
        .fetch_one(db)
        .await;

    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn create_user(
    username: &str,
    email: &str,
    password_hash: &str,
    db: &PgPool,
) -> Result<(), HTTPError> {
    /// Create a new user in DB
    ///
    /// # Arguments
    ///  username: &str - The username of the user
    ///  email: &str - The email of the user
    ///  password_hash: &str - The password hash of the user
    ///
    /// # Returns
    ///  Result<(), HTTPError> - The result of the operation
    let uid = Uuid::new_v4();
    let verification_token = random_string(8);

    if check_username(username, db).await || check_email(email, db).await {
        return Err(HTTPError::Conflict);
    }

    let result = sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, verification_token) VALUES ($1, $2, $3, $4, $5)",
        uid,
        username,
        email,
        password_hash,
        verification_token
    ).bind(uid).bind(username).bind(email).bind(password_hash).bind(verification_token).execute(db).await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(HTTPError::from(e)),
    }
}

pub async fn delete_user(uid: &Uuid, db: &PgPool) -> Result<(), HTTPError> {
    /// Delete a user from DB
    ///
    /// # Arguments
    ///  uid: &Uuid - The user id of the user
    ///  db: &PgPool - The database connection pool
    ///
    /// # Returns
    ///  Result<(), sqlx::Error> - The result of the operation
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", uid)
        .bind(uid)
        .execute(db)
        .await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(HTTPError::from(e)),
    }
}
pub async fn get_hash(username: &str, db: &PgPool) -> Result<(Uuid, String), sqlx::Error> {
    /// Get the user's id and password hash from the DB
    ///
    /// # Arguments
    ///   username: &str - The username of the user
    ///   db: &PgPool - The database connection pool
    ///
    /// # Returns
    ///  (Uuid, String) - The user's id and password hash
    let row = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_one(db)
    .await?;

    Ok((row.id, row.password_hash))
}
