//! tower_jsonapi_client is a library for building strongly typed REST clients, with built-in capabilites
//! for authentication, various request and response types and pagination.
//!
//! Originally inspired by [ring-api](https://github.com/H2CO3/ring_api)
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
