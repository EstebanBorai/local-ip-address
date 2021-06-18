use crate::Error;
use libc::{getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, strlen, AF_INET, AF_INET6};
use std::env;
use std::ffi::CStr;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
/// `ifaddrs` struct raw pointer alias
type IfAddrsPtr = *mut *mut ifaddrs;

pub fn impl_find_af_inet() -> Result<Vec<(String, IpAddr)>, Error> {
    let ifaddrs_size = mem::size_of::<IfAddrsPtr>();

    unsafe {
        let myaddr: IfAddrsPtr = libc::malloc(ifaddrs_size) as IfAddrsPtr;
        let getifaddrs_result = getifaddrs(myaddr);

        if getifaddrs_result != 0 {
            // an error ocurred on getifaddrs
            return Err(Error::GetIfAddrsError(getifaddrs_result));
        }

        let mut interfaces: Vec<(String, IpAddr)> = Vec::new();
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
unsafe fn get_ifa_name(ifa: *mut *mut ifaddrs) -> Result<String, Error> {
    let str = (*(*ifa)).ifa_name as *mut u8;
    let len = strlen(str as *const i8);
    let slice = std::slice::from_raw_parts(str, len);
    match String::from_utf8(slice.to_vec()) {
        Ok(s) => Ok(s),
        Err(_e) => Err(Error::IntAddrNameParseError(_e)),
    }
}
