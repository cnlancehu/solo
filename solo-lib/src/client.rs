//! # Solo http client
//! Solo uses reqwest as its http request processing module
//!
//! This module provides some simple functions to create a reqwest client
//!
//! ### Example
//! ```rust
//! use std::time::Duration;
//!
//! use solo_lib::client::new_builder;
//!
//! let client = new_builder()
//!     .no_proxy()
//!     .timeout(Duration::from_secs(10))
//!     .build()
//!     .unwrap();
//! ```

use reqwest::{Client, ClientBuilder};

/// Create a new reqwest client
///
/// Equals to
/// ```rust
/// reqwest::Client::new()
/// ```
pub fn new() -> Client {
    Client::new()
}

/// Create a new reqwest client builder
///
/// Equals to
/// ```rust
/// reqwest::ClientBuilder::new()
/// ```
pub fn new_builder() -> ClientBuilder {
    ClientBuilder::new().user_agent("Solo")
}
