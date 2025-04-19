//! # Error types for the Solo SDK
//!
//! This module provides a unified error handling mechanism for cloud service providers.
//! It standardizes error responses from different cloud vendors into a consistent format,
//! making it easier to handle errors across different cloud platforms.

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for the Solo SDK
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub struct SdkError {
    /// Request ID
    pub request_id: String,
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

impl Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SdkError: request_id={}, code={}, message={}",
            self.request_id, self.code, self.message
        )
    }
}
