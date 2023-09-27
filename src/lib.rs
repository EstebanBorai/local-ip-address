/*!
# Local IP Address

Retrieve system's local IP address and Network Interfaces/Adapters on
Linux, Windows, and macOS (and other BSD-based systems).

## Usage

Get the local IP address of your system by executing the `local_ip` function:

```rust
use local_ip_address::local_ip;

let my_local_ip = local_ip();

if let Ok(my_local_ip) = my_local_ip {
    println!("This is my local IP address: {:?}", my_local_ip);
} else {
    println!("Error getting local IP: {:?}", my_local_ip);
}
```

Retrieve all the available network interfaces from both, the `AF_INET` and
the `AF_INET6` family by executing the `list_afinet_netifas` function:

```rust
use local_ip_address::list_afinet_netifas;

let network_interfaces = list_afinet_netifas();

if let Ok(network_interfaces) = network_interfaces {
    for (name, ip) in network_interfaces.iter() {
        println!("{}:\t{:?}", name, ip);
    }
} else {
    println!("Error getting network interfaces: {:?}", network_interfaces);
}
```

Underlying approach on retrieving network interfaces or the local IP address
may differ based on the running operative system.

OS | Approach
--- | ---
Linux | Establishes a Netlink socket interchange to retrieve network interfaces
BSD-based & Android | Uses of `getifaddrs` to retrieve network interfaces
Windows | Consumes Win32 API's to retrieve the network adapters table

Supported BSD-based systems include:
  - macOS
  - FreeBSD
  - OpenBSD
  - NetBSD
  - DragonFly
*/

use std::net::IpAddr;

mod error;

pub use error::Error;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux::*;

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "android",
    target_os = "ios",
))]
pub mod unix;

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "android",
    target_os = "ios",
))]
pub use crate::unix::*;

#[cfg(target_family = "windows")]
pub mod windows;
#[cfg(target_family = "windows")]
pub use crate::windows::*;

/// Retrieves the local IPv4 address of the machine in the local network from
/// the `AF_INET` family.
///
/// A different approach is taken based on the operative system.
///
/// For linux based systems the Netlink socket communication is used to
/// retrieve the local network interface.
///
/// For BSD-based systems the `getifaddrs` approach is taken using `libc`
///
/// For Windows systems Win32's IP Helper is used to gather the Local IP
/// address
pub fn local_ip() -> Result<IpAddr, Error> {
    #[cfg(target_os = "linux")]
    {
        crate::linux::local_ip()
    }

    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
    ))]
    {
        let ifas = crate::unix::list_afinet_netifas_info()?;

        ifas.into_iter()
            .find_map(|ifa| {
                if !ifa.is_loopback && ifa.addr.is_ipv4() {
                    Some(ifa.addr)
                } else {
                    None
                }
            })
            .ok_or(Error::LocalIpAddressNotFound)
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Networking::WinSock::AF_INET;

        let ip_addresses = crate::windows::list_local_ip_addresses(AF_INET)?;

        ip_addresses
            .into_iter()
            .find(|ip_address| matches!(ip_address, IpAddr::V4(_)))
            .ok_or(Error::LocalIpAddressNotFound)
    }

    // A catch-all case to error if not implemented for OS
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "android",
        target_os = "ios",
    )))]
    {
        Err(Error::PlatformNotSupported(
            std::env::consts::OS.to_string(),
        ))
    }
}

/// Retrieves the local IPv6 address of the machine in the local network from
/// the `AF_INET6` family.
///
/// A different approach is taken based on the operative system.
///
/// For linux based systems the Netlink socket communication is used to
/// retrieve the local network interface.
///
/// For BSD-based systems the `getifaddrs` approach is taken using `libc`
///
/// For Windows systems Win32's IP Helper is used to gather the Local IP
/// address
pub fn local_ipv6() -> Result<IpAddr, Error> {
    #[cfg(target_os = "linux")]
    {
        crate::linux::local_ipv6()
    }

    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
    ))]
    {
        let ifas = crate::unix::list_afinet_netifas_info()?;

        ifas.into_iter()
            .find_map(|ifa| {
                if !ifa.is_loopback && ifa.addr.is_ipv6() {
                    Some(ifa.addr)
                } else {
                    None
                }
            })
            .ok_or(Error::LocalIpAddressNotFound)
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Networking::WinSock::AF_INET6;

        let ip_addresses = crate::windows::list_local_ip_addresses(AF_INET6)?;

        ip_addresses
            .into_iter()
            .find(|ip_address| matches!(ip_address, IpAddr::V6(_)))
            .ok_or(Error::LocalIpAddressNotFound)
    }

    // A catch-all case to error if not implemented for OS
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "android",
        target_os = "ios",
    )))]
    {
        Err(Error::PlatformNotSupported(
            std::env::consts::OS.to_string(),
        ))
    }
}

// A catch-all function to error if not implemented for OS
#[cfg(not(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "android",
    target_os = "ios",
)))]
pub fn list_afinet_netifas() -> Result<Vec<(String, IpAddr)>, Error> {
    Err(Error::PlatformNotSupported(
        std::env::consts::OS.to_string(),
    ))
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn find_local_ip() {
        let my_local_ip = local_ip();

        assert!(matches!(my_local_ip, Ok(IpAddr::V4(_))));
        println!("Linux 'local_ip': {:?}", my_local_ip);
    }

    #[test]
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
    ))]
    fn find_local_ip() {
        let my_local_ip = local_ip();

        assert!(matches!(my_local_ip, Ok(IpAddr::V4(_))));
        println!("Unix 'local_ip': {:?}", my_local_ip);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn find_local_ip() {
        let my_local_ip = local_ip();

        assert!(matches!(my_local_ip, Ok(IpAddr::V4(_))));
        println!("Windows 'local_ip': {:?}", my_local_ip);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn find_network_interfaces() {
        let network_interfaces = list_afinet_netifas();

        assert!(network_interfaces.is_ok());
        assert!(!network_interfaces.unwrap().is_empty());
    }

    #[test]
    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
    ))]
    fn find_network_interfaces() {
        let network_interfaces = list_afinet_netifas();

        assert!(network_interfaces.is_ok());
        assert!(!network_interfaces.unwrap().is_empty());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn find_network_interfaces() {
        let network_interfaces = list_afinet_netifas();

        assert!(network_interfaces.is_ok());
        assert!(!network_interfaces.unwrap().is_empty());
    }
}
