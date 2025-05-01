//! # Solo Library
//!
//! Library for [the Solo app](https://github.com/cnlancehu/solo).
//!
//! A lightweight tool that dynamically secures server ports.
//!
//! Solo automatically detects the current public IP, dynamically adjusts firewall rules through the cloud provider's API, and locks the source IP of the specified port (such as SSH/22) to the current IP.
//!
//! ## Supported Platforms
//! | Service Provider |         Products          |
//! | :----: | :---------------------: |
//! | Tencent Cloud | Cloud Server and Lighthouse |
//! | Aliyun | Elastic Compute Service (ECS) and Simple Application Server (SWAS) |

pub mod client;
mod error;
pub use error::*;
pub mod sdk;
#[doc(hidden)]
pub(crate) mod util;
