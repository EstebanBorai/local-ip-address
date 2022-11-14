/*!
# Local IP Address

Retrieve system's local IP address and Network Interfaces/Adapters on
Linux, macOS and Windows.

## Usage

Get the local IP address of your system by executing the `local_ip` function:

```rust
use local_ip_address::local_ip;

let my_local_ip = local_ip().unwrap();

println!("This is my local IP address: {:?}", my_local_ip);
```

Retrieve all the available network interfaces from both, the `AF_INET` and
the `AF_INET6` family by executing the `list_afinet_netifas` function:

```rust
use local_ip_address::list_afinet_netifas;

let network_interfaces = list_afinet_netifas().unwrap();

for (name, ip) in network_interfaces.iter() {
    println!("{}:\t{:?}", name, ip);
}
```

Underlying approach on retrieving network interfaces or the local IP address
may differ based on the running operative system.

OS | Approach
--- | ---
Linux | Establishes a Netlink socket interchange to retrieve network interfaces
macOS | Uses of `getifaddrs` to retrieve network interfaces
Windows | Consumes Win32 API's to retrieve the network adapters table

*/
use std::net::IpAddr;

mod error;

pub use error::Error;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux::*;

#[cfg(any(target_os = "macos", target_os = "openbsd"))]
pub mod macos;
#[cfg(any(target_os = "macos", target_os = "openbsd"))]
pub use crate::macos::*;

#[cfg(target_family = "windows")]
pub mod windows;
#[cfg(target_family = "windows")]
pub use crate::windows::*;

/// Retrieves the local ip address of the machine in the local network from
/// the `AF_INET` family.
///
/// A different approach is taken based on the operative system.
///
/// For linux based systems the Netlink socket communication is used to
/// retrieve the local network interface.
///
/// For macOS systems the `getifaddrs` approach is taken using `libc`
///
/// For Windows systems Win32's IP Helper is used to gather the Local IP
/// address
pub fn local_ip() -> Result<IpAddr, Error> {
    #[cfg(target_os = "linux")]
    {
        crate::linux::local_ip()
    }

    #[cfg(any(target_os = "macos", target_os = "openbsd"))]
    {
        use std::env;

        let ifas = crate::macos::list_afinet_netifas()?;

        if let Some((_, ipaddr)) = find_ifa(ifas, "en0") {
            return Ok(ipaddr);
        }

        Err(Error::PlatformNotSupported(env::consts::OS.to_string()))
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        use windows_sys::Win32::Networking::WinSock::AF_INET;

        let ip_addresses = crate::windows::list_local_ip_addresses(AF_INET)?;

        ip_addresses
            .into_iter()
            .find(|ip_address| matches!(ip_address, IpAddr::V4(_)))
            .ok_or_else(|| Error::PlatformNotSupported(env::consts::OS.to_string()))
    }
}

/// Finds the network interface with the provided name in the vector of network
/// interfaces provided
pub fn find_ifa(ifas: Vec<(String, IpAddr)>, ifa_name: &str) -> Option<(String, IpAddr)> {
    ifas.into_iter()
        .find(|(name, ipaddr)| name == ifa_name && matches!(ipaddr, IpAddr::V4(_)))
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
    #[cfg(target_os = "linux")]
    fn find_network_interfaces() {
        let network_interfaces = list_afinet_netifas();

        assert!(network_interfaces.is_ok());
        assert!(network_interfaces.unwrap().len() >= 1);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn find_network_interfaces() {
        let network_interfaces = list_afinet_netifas();

        assert!(network_interfaces.is_ok());
        assert!(network_interfaces.unwrap().len() >= 1);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn find_network_interfaces() {
        let network_interfaces = list_afinet_netifas();

        assert!(network_interfaces.is_ok());
        assert!(network_interfaces.unwrap().len() >= 1);
    }
}
