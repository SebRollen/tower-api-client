//! tower_api_client is a library for building strongly typed API clients, with built-in capabilites
//! for authentication, various request and response types and pagination.
mod client;
mod error;
pub mod pagination;
mod request;

pub use client::{Client, ServiceExt};
pub use error::Error;
pub use hyper::header;
pub use hyper::Method;
pub use hyper::StatusCode;
pub use request::*;
