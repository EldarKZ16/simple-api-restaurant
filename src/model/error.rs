use std::fmt::Display;
use std::error::Error;
use warp::reject::Reject;

#[derive(Debug)]
pub enum GeneralError {
    NotFound,
    LockFailed(String),
    ValidationFailed(String)
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match *self {
           GeneralError::NotFound => write!(f, "NOT_FOUND"),
           GeneralError::LockFailed(ref msg) => write!(f, "Lock failed, reason: {}", msg),
           GeneralError::ValidationFailed(ref msg) => write!(f, "Validation failed, reason: {}", msg)
       }
    }
}

impl Error for GeneralError {}

#[derive(Debug)]
pub struct GeneralErrorRejection {
    pub err: GeneralError,
}

impl Reject for GeneralErrorRejection {}

impl GeneralError {
    pub fn reject(self) -> warp::Rejection {
        warp::reject::custom(GeneralErrorRejection { err: self })
    }
}