//! # Solo Library
//!
//! Library for [the Solo app](https://github.com/cnlancehu/solo).
//! 
//! [Documentation](https://solo.lance.fun/)
//!
//! A lightweight port protection tool.
//!
//! ## Supported Platforms
//! 
//! |   Provider    |                      Products                      |
//! | :-----------: | :------------------------------------------------: |
//! | Tencent Cloud |         Cloud Virtual Machine, Lighthouse          |
//! |    Aliyun     | Elastic Compute Service, Simple Application Server |

pub mod client;
mod error;
pub use error::*;
pub mod sdk;
#[doc(hidden)]
pub(crate) mod util;
