use crate::generic_handler_err;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("username or email is already taken")]
    EmailOrUsernameExists,
    #[error("{0}")]
    Validation(#[from] ValidationError),
    #[error("Wrong password")]
    WrongPassword,
    #[error("User does not exist")]
    UserNotFound,
    #[error("Internal error: {0:#?}")]
    Internal(Box<dyn std::error::Error + Sync + Send>),
    #[error("Could not create session")]
    Session,
    #[error("Unauthorized or the session has expired")]
    Unauthorized,
    #[error("{0}")]
    BadForm(#[from] actix_web::Error),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("The Passwords are not the same")]
    PasswordMisMatch,
    #[error("Invalid email address")]
    InvalidEmail,
    #[error("The Password is to short. It should be a least 4 chars long")]
    PasswordShort(String),
}

generic_handler_err!(AuthError, AuthError::Internal);
