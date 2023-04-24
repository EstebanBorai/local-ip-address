use std::alloc::{alloc, dealloc, Layout};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use libc::{
    getifaddrs, strlen, c_char, ifaddrs, sockaddr_in, sockaddr_in6, AF_INET, AF_INET6, IFF_LOOPBACK,
};

use crate::Error;

/// `ifaddrs` struct raw pointer alias
type IfAddrsPtr = *mut *mut ifaddrs;

/// Perform a search over the system's network interfaces using `getifaddrs`,
/// retrieved network interfaces belonging to both socket address families
/// `AF_INET` and `AF_INET6` are retrieved along with the interface address name.
///
/// # Example
///
/// ```
/// use std::net::IpAddr;
/// use local_ip_address::list_afinet_netifas;
///
/// let ifas = list_afinet_netifas().unwrap();
///
/// if let Some((_, ipaddr)) = ifas
/// .iter()
/// .find(|(name, ipaddr)| (*name == "en0" || *name == "epair0b") && matches!(ipaddr, IpAddr::V4(_))) {
///     // This is your local IP address: 192.168.1.111
///     println!("This is your local IP address: {:?}", ipaddr);
/// }
/// ```
pub fn list_afinet_netifas() -> Result<Vec<(String, IpAddr)>, Error> {
    match list_afinet_netifas_info() {
        Ok(interfaces) => Ok(interfaces
            .iter()
            .map(|i| (i.iname.clone(), i.addr))
            .collect()),
        Err(e) => Err(e),
    }
}

pub(crate) struct AfInetInfo {
    pub addr: IpAddr,
    pub iname: String,
    pub is_loopback: bool,
}

// Internal method to list AF_INET info in a struct.  This method is used by
// list_afiinet_netifas and local_ip,
pub(crate) fn list_afinet_netifas_info() -> Result<Vec<AfInetInfo>, Error> {
    unsafe {
        let layout = Layout::new::<IfAddrsPtr>();
        let ptr = alloc(layout);
        let myaddr = ptr as IfAddrsPtr;
        let getifaddrs_result = getifaddrs(myaddr);

        if getifaddrs_result != 0 {
            // an error occurred on getifaddrs
            return Err(Error::StrategyError(format!(
                "GetIfAddrs returned error: {}",
                getifaddrs_result
            )));
        }

        let mut interfaces: Vec<AfInetInfo> = Vec::new();
        let ifa = myaddr;

        // An instance of `ifaddrs` is build on top of a linked list where
        // `ifaddrs.ifa_next` represent the next node in the list.
        //
        // To find the relevant interface address walk over the nodes of the
        // linked list looking for interface address which belong to the socket
        // address families AF_INET (IPv4) and AF_INET6 (IPv6)
        loop {
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
                        // on CPU endianness to avoid having twisted IP addresses
                        //
                        // refer: https://github.com/rust-lang/rust/issues/48819
                        ip_addr = Ipv4Addr::from(in_addr.s_addr.swap_bytes());
                    }

                    interfaces.push(AfInetInfo {
                        addr: IpAddr::V4(ip_addr),
                        iname: get_ifa_name(ifa)?,
                        is_loopback: is_loopback_addr(ifa),
                    });
                }
                // AF_INET6 IPv6 protocol implementation
                AF_INET6 => {
                    let interface_address = ifa_addr;
                    let socket_addr_v6: *mut sockaddr_in6 = interface_address as *mut sockaddr_in6;
                    let in6_addr = (*socket_addr_v6).sin6_addr;
                    let ip_addr = Ipv6Addr::from(in6_addr.s6_addr);

                    interfaces.push(AfInetInfo {
                        addr: IpAddr::V6(ip_addr),
                        iname: get_ifa_name(ifa)?,
                        is_loopback: is_loopback_addr(ifa),
                    });
                }
                _ => {}
            }

            // Check if we are at the end of our network interface list
            *ifa = (**ifa).ifa_next;
            if (*ifa).is_null() {
                break;
            }
        }

        dealloc(ptr, layout);
        Ok(interfaces)
    }
}

/// Retrieves the name of a interface address
unsafe fn get_ifa_name(ifa: *mut *mut ifaddrs) -> Result<String, Error> {
    let str = (*(*ifa)).ifa_name;
    let len = strlen(str as *const c_char);
    let slice = std::slice::from_raw_parts(str as *mut u8, len);
    match String::from_utf8(slice.to_vec()) {
        Ok(s) => Ok(s),
        Err(e) => Err(Error::StrategyError(format!(
            "Failed to retrieve interface name. The name is not a valid UTF-8 string. {}",
            e
        ))),
    }
}

/// Determines if an interface address is a loopback address
unsafe fn is_loopback_addr(ifa: *mut *mut ifaddrs) -> bool {
    let iflags = (*(*ifa)).ifa_flags as i32;
    (iflags & IFF_LOOPBACK) != 0
}
