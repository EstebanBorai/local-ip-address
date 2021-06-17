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

use std::str::Utf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error occured building a `&str` from a C string when
    /// parsing the name of a interface address instance
    #[error("Failed to read interface address name. `{0}`")]
    IntAddrNameParseError(Utf8Error),
    /// An error ocurred calling `getifaddrs`
    #[error("Execution of getifaddrs had error result. getifaddrs returned `{0}`")]
    GetIfAddrsError(i32),
    /// The current platform is not supported
    #[error("The current platform `{0}` is not supported")]
    PlatformNotSupported(String),
}

#[cfg(target_family = "unix")]
pub mod unix;
#[cfg(target_family = "windows")]
pub mod windows;
