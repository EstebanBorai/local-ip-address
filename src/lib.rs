use libc::{getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, AF_INET, AF_INET6};
use std::ffi::CStr;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::Utf8Error;

/// `ifaddrs` struct raw pointer alias
type IfAddrsPtr = *mut *mut ifaddrs;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to read interface address name. `{0}`")]
    IntAddrNameParseError(Utf8Error),
    /// An error ocurred calling `getifaddrs`
    #[error("Execution of getifaddrs had error result. getifaddrs returned `{0}`")]
    GetIfAddrsError(i32),
}

/// Perform a search over the system's network interfaces using `getifaddrs`,
/// retrieved network interfaces belonging to both socket address families
/// `AF_INET` and `AF_INET6` are retrieved along with the interface address name.
///
/// # Example
///
/// ```
/// use std::net::IpAddr;
/// use local_ip_address::find_af_inet;
///
/// let ifas = find_af_inet().unwrap();
///
/// if let Some((_, ipaddr)) = ifas
/// .iter()
/// .find(|(name, ipaddr)| *name == "en0" && matches!(ipaddr, IpAddr::V4(_))) {
///     // This is your local IP address: 192.168.1.111
///     println!("This is your local IP address: {:?}", ipaddr);
/// }
/// ```
pub fn find_af_inet<'a>() -> Result<Vec<(&'a str, IpAddr)>, Error> {
    let ifaddrs_size = mem::size_of::<IfAddrsPtr>();

    unsafe {
        let myaddr: IfAddrsPtr = libc::malloc(ifaddrs_size) as IfAddrsPtr;
        let getifaddrs_result = getifaddrs(myaddr);

        if getifaddrs_result != 0 {
            // an error ocurred on getifaddrs
            return Err(Error::GetIfAddrsError(getifaddrs_result));
        }

        let mut interfaces: Vec<(&'a str, IpAddr)> = Vec::new();
        let ifa = myaddr;

        // An instance of `ifaddrs` is build on top of a linked list where
        // `ifaddrs.ifa_next` represent the next node in the list.
        //
        // To find the relevant interface address walk over the nodes of the
        // linked list looking for interface address which belong to the socket
        // address families AF_INET (IPv4) and AF_INET6 (IPv6)
        while !(**ifa).ifa_next.is_null() {
            let ifa_addr = (**ifa).ifa_addr;

            match (*ifa_addr).sa_family as i32 {
                // AF_INET IPv4 protocol implementation
                AF_INET => {
                    let interface_address = ifa_addr;
                    let socket_addr_v4: *mut sockaddr_in = interface_address as *mut sockaddr_in;
                    let in_addr = (*socket_addr_v4).sin_addr;
                    let mut ip_addr = Ipv4Addr::from(in_addr.s_addr);

                    if cfg!(target_endian = "little") {
                        // due to a difference on how bytes are arranged on a
                        // single word of memory by the CPU, swap bytes based
                        // on CPU endianess to avoid having twisted IP addresses
                        //
                        // refer: https://github.com/rust-lang/rust/issues/48819
                        ip_addr = Ipv4Addr::from(in_addr.s_addr.swap_bytes());
                    }

                    let name = get_ifa_name(ifa)?;

                    interfaces.push((name, IpAddr::V4(ip_addr)));

                    *ifa = (**ifa).ifa_next;
                    continue;
                }
                // AF_INET6 IPv6 protocol implementation
                AF_INET6 => {
                    let interface_address = ifa_addr;
                    let socket_addr_v6: *mut sockaddr_in6 = interface_address as *mut sockaddr_in6;
                    let in6_addr = (*socket_addr_v6).sin6_addr;
                    let ip_addr = Ipv6Addr::from(in6_addr.s6_addr);
                    let name = get_ifa_name(ifa)?;

                    interfaces.push((name, IpAddr::V6(ip_addr)));

                    *ifa = (**ifa).ifa_next;
                    continue;
                }
                _ => {
                    *ifa = (**ifa).ifa_next;
                    continue;
                }
            }
        }

        Ok(interfaces)
    }
}

/// Retrieves the name of a interface address
unsafe fn get_ifa_name<'a>(ifa: *mut *mut ifaddrs) -> Result<&'a str, Error> {
    CStr::from_ptr((**ifa).ifa_name)
        .to_str()
        .map_err(Error::IntAddrNameParseError)
}
