//! `local-ip-address` is a wrapper on `getifaddrs` which retrieves host's
//! network interfaces.
//!
//! Handy functions are provided such as `local_ip` which retrieve the local IP
//! address based on the host system
//!
//! ```
//! use std::net::IpAddr;
//! use local_ip_address::local_ip;
//!
//! assert!(matches!(local_ip().unwrap(), IpAddr));
//! ```
//!
//! You are able to iterate over a vector of tuples where the first element of
//! the tuple is the name of the network interface and the second is the IP
//! address.
//!
//! ```
//! use std::net::IpAddr;
//! use local_ip_address::find_af_inet;
//!
//! let ifas = find_af_inet().unwrap();
//!
//! if let Some((_, ipaddr)) = ifas
//! .iter()
//! .find(|(name, ipaddr)| *name == "en0" && matches!(ipaddr, IpAddr::V4(_))) {
//!     println!("This is your local IP address: {:?}", ipaddr);
//!     // This is your local IP address: 192.168.1.111
//!     assert!(matches!(ipaddr, IpAddr));
//! }
//! ```
//!
use std::string::FromUtf8Error;
use std::{env, net::IpAddr};
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occured building a `&str` from a C string when
    /// parsing the name of a interface address instance
    #[error("Failed to read interface address name. `{0}`")]
    IntAddrNameParseError(FromUtf8Error),
    /// An error ocurred calling `getifaddrs`
    #[error("Execution of getifaddrs had error result. getifaddrs returned `{0}`")]
    GetIfAddrsError(i32),
    /// The current platform is not supported
    #[error("The current platform `{0}` is not supported")]
    PlatformNotSupported(String),
    #[error("GetIpAddrTableError")]
    GetAdaptersAddresses(u32),
}

#[cfg(target_family = "unix")]
pub mod unix;
#[cfg(target_family = "unix")]
pub use crate::unix::*;

#[cfg(target_family = "windows")]
pub mod windows;
#[cfg(target_family = "windows")]
pub use crate::windows::*;

/// Finds the network interface with the provided name in the vector of network
/// interfaces provided
pub fn find_ifa(ifas: Vec<(String, IpAddr)>, ifa_name: &str) -> Option<(String, IpAddr)> {
    ifas.into_iter()
        .find(|(name, ipaddr)| name == ifa_name && matches!(ipaddr, IpAddr::V4(_)))
}

/// Retrieves the local ip address for the current operative system
pub fn local_ip() -> Result<IpAddr, Error> {
    let ifas = find_af_inet()?;

    #[cfg(target_os = "macos")]
    const DEFAULT_IF_NAME: &str = "en0";
    #[cfg(target_os = "linux")]
    const DEFAULT_IF_NAME: &str = "wlp2s0";
    #[cfg(target_os = "windows")]
    const DEFAULT_IF_NAME: &str = "Ethernet";

    if let Some((_, ipaddr)) = find_ifa(ifas, DEFAULT_IF_NAME) {
        return Ok(ipaddr);
    }

    Err(Error::PlatformNotSupported(env::consts::OS.to_string()))
}
