//! # Solo SDKs
//! This module contains the SDKs for the various cloud providers.
//!
//! ### Notice
//! In some cases, the firewall rules supports both IPv4 and IPv6 addresses.
//!
//! So when you are asked to provide both IPv4 and IPv6 addresses,
//! like in [`aliyun::ecs::go`] function, checkout the info below:
//!
//! If an IP address is left empty, then no replacement is performed for that address.
//! For example, if you only provide an IPv4 address, it will replace the original IPv4
//! firewall rule and leave the IPv6 rule as is. Similarly, providing an IPv6 address
//! replaces only the IPv6 rule.
//!
//! Note that you can also provide both IPv4 and IPv6 addresses to replace both rules.

pub mod aliyun;
pub mod qcloud;
