//! # Solo
//!
//! Security made simple.
//!
//! A simple and secure port management library.
//!
//! Automatically restrict port access to your IP address with minimal configuration.
//!
//! ## Overview
//! Solo helps secure your services by:
//! 1. **Auto-detecting your current IP address**
//! 2. **Configuring cloud firewall rules to only allow access from your IP**
//!
//! ## Supported Platforms
//! Solo integrates with major cloud providers:
//! - Alibaba Cloud
//!   - Elastic Compute Service (ECS)
//!   - Simple Application Server (SWAS)
//! - Tencent Cloud
//!   - Virtual Private Cloud (VPC)
//!   - Lighthouse

pub mod client;
mod error;
pub use error::*;
pub mod sdk;
#[doc(hidden)]
pub(crate) mod util;
