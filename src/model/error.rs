use std::fmt::Display;
use std::error::Error;
use warp::reject::Reject;

#[derive(Debug)]
pub enum OrderError {
    NotFound,
    LockFailed(String),
    ValidationFailed(String)
}

impl Display for OrderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match *self {
           OrderError::NotFound => write!(f, "NOT_FOUND"),
           OrderError::LockFailed(ref msg) => write!(f, "Lock failed, reason: {}", msg),
           OrderError::ValidationFailed(ref msg) => write!(f, "Validation failed, reason: {}", msg)
       }
    }
}

impl Error for OrderError {}

#[derive(Debug)]
pub struct OrderErrorRejection {
    pub err: OrderError,
}

impl Reject for OrderErrorRejection {}

impl OrderError {
    pub fn reject(self) -> warp::Rejection {
        warp::reject::custom(OrderErrorRejection { err: self })
    }
}