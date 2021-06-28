/*!
Retrieve system's local IP address on Rust applications using `getifaddrs`
on Unix based systems and Win32's `GetAdaptersAddresses` on Windows

A wrapper on `getifaddrs` for Unix based systems and `GetAdaptersAddresses` on
Windows which retrieves host's network interfaces.

Handy functions are provided such as `local_ip` which retrieve the local IP
address based on the host system

```ignore
use std::net::IpAddr;
use local_ip_address::local_ip;

assert!(matches!(local_ip().unwrap(), IpAddr));
```

Is important to note that `local_ip` attempts to find commonly used network
interface names on most of the systems. As of now only support for macOS and
Windows is granted.

Network interface names may change on different Linux distribution and hardware
units. If your solution runs on different platforms its recommended to consume
the `find_af_inet` function and search for the expected interface name instead
of using `local_ip` directly.

Help improve the support for multiple systems on this crate by opening a pull
request or issue on [GitHub](https://github.com/EstebanBorai/local-ip-address).

```ignore
use std::net::IpAddr;
use local_ip_address::find_af_inet;

let ifas = find_af_inet().unwrap();

if let Some((_, ipaddr)) = ifas
    .iter()
    .find(|(name, ipaddr)| *name == "en0" && matches!(ipaddr, IpAddr::V4(_))) {
    println!("This is your local IP address: {:?}", ipaddr);
    // This is your local IP address: 192.168.1.111
    assert!(matches!(ipaddr, IpAddr));
}
```

*/
use std::net::IpAddr;
use std::string::FromUtf8Error;

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
    #[error("Netlink IO Error `{0}`")]
    NetlinkIOError(String),
    #[error("Netlink: Socket Message Error `{0}`")]
    NetlinkSendMessageError(String),
    #[error("Netlink: Failed to find local IP address")]
    NetlinkFailedToFindLocalIp,
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux::*;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use crate::macos::*;

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
    #[cfg(target_os = "linux")]
    {
        use crate::linux::local_ip;

        local_ip()
    }

    #[cfg(target_os = "macos")]
    {
        use std::env;

        let ifas = crate::macos::find_af_inet()?;

        if let Some((_, ipaddr)) = find_ifa(ifas, "en0") {
            return Ok(ipaddr);
        }

        Err(Error::PlatformNotSupported(env::consts::OS.to_string()))
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        let ifas = find_af_inet()?;

        if let Some((_, ipaddr)) = find_ifa(ifas, "Ethernet") {
            return Ok(ipaddr);
        }

        Err(Error::PlatformNotSupported(env::consts::OS.to_string()))
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn find_local_ip() {
        let my_local_ip = local_ip().unwrap();

        assert!(matches!(my_local_ip, IpAddr::V4(_)));
        println!("Linux 'local_ip': {:?}", my_local_ip);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn find_local_ip() {
        let my_local_ip = local_ip().unwrap();

        assert!(matches!(my_local_ip, IpAddr::V4(_)));
        println!("macOS 'local_ip': {:?}", my_local_ip);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn find_local_ip() {
        let my_local_ip = local_ip().unwrap();

        assert!(matches!(my_local_ip, IpAddr::V4(_)));
        println!("Windows 'local_ip': {:?}", my_local_ip);
    }

    #[test]
    #[cfg(target_os = "macos")]
    #[cfg(target_os = "windows")]
    fn find_network_interfaces() {
        let network_interfaces = find_af_inet();

        assert!(network_interfaces.is_ok());
        assert!(network_interfaces.unwrap().len() >= 1);
    }
}
