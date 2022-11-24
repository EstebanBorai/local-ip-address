use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use neli::attr::Attribute;
use neli::consts::nl::{NlmF, NlmFFlags};
use neli::consts::socket::NlFamily;
use neli::consts::rtnl::{Ifa, IfaFFlags, RtAddrFamily, RtScope, Rtm};
use neli::nl::{NlPayload, Nlmsghdr};
use neli::rtnl::Ifaddrmsg;
use neli::socket::NlSocketHandle;
use neli::types::RtBuffer;
use libc::{freeifaddrs, getifaddrs, ifaddrs, sockaddr_in, sockaddr_in6, strlen, AF_INET, AF_INET6};

use crate::Error;

fn make_ifaddrmsg() -> Ifaddrmsg {
    Ifaddrmsg {
        ifa_family: RtAddrFamily::Inet,
        ifa_prefixlen: 0,
        ifa_flags: IfaFFlags::empty(),
        ifa_scope: 0,
        ifa_index: 0,
        rtattrs: RtBuffer::new(),
    }
}

fn make_netlink_message(ifaddrmsg: NlPayload<Ifaddrmsg>) -> Nlmsghdr<Rtm, NlPayload<Ifaddrmsg>> {
    Nlmsghdr::new(
        None,
        Rtm::Getaddr,
        NlmFFlags::new(&[NlmF::Request, NlmF::Root]),
        None,
        None,
        NlPayload::Payload(ifaddrmsg),
    )
}

/// Retrieves the local IP address fo this system
pub fn local_ip() -> Result<IpAddr, Error> {
    let mut netlink_socket = NlSocketHandle::connect(NlFamily::Route, None, &[])
        .map_err(|err| Error::StrategyError(err.to_string()))?;
    let ifaddrmsg = make_ifaddrmsg();
    let netlink_payload = NlPayload::Payload(ifaddrmsg);
    let netlink_message = make_netlink_message(netlink_payload);

    netlink_socket
        .send(netlink_message)
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    let mut addrs = Vec::<Ipv4Addr>::with_capacity(1);

    for response in netlink_socket.iter(false) {
        let header: Nlmsghdr<_, Ifaddrmsg> = response.map_err(|_| {
            Error::StrategyError(String::from(
                "An error ocurred retrieving Netlink's socket response",
            ))
        })?;

        if let NlPayload::Empty = header.nl_payload {
            continue;
        }

        if header.nl_type != Rtm::Newaddr.into() {
            return Err(Error::StrategyError(String::from(
                "The Netlink header type is not the expected",
            )));
        }

        let p = header.get_payload().map_err(|_| {
            Error::StrategyError(String::from(
                "An error ocurred getting Netlink's header payload",
            ))
        })?;

        if RtScope::from(p.ifa_scope) != RtScope::Universe {
            continue;
        }

        for rtattr in p.rtattrs.iter() {
            if rtattr.rta_type == Ifa::Local {
                addrs.push(Ipv4Addr::from(u32::from_be(
                    rtattr.get_payload_as::<u32>().map_err(|_| {
                        Error::StrategyError(String::from(
                            "An error ocurred retrieving Netlink's route payload attribute",
                        ))
                    })?,
                )));
            }
        }
    }

    if let Some(local_ip) = addrs.first() {
        let ipaddr = IpAddr::V4(local_ip.to_owned());

        return Ok(ipaddr);
    }

    Err(Error::LocalIpAddressNotFound)
}

struct IfAddrList {
    ptr: *mut ifaddrs,
}

impl IfAddrList {
    fn new() -> Result<IfAddrList, Error> {
        unsafe {
            let mut ptr: *mut ifaddrs = std::ptr::null_mut();
            let res = getifaddrs(&mut ptr);
            if res != 0 || ptr.is_null() {
                return Err(Error::StrategyError(format!(
                    "getifaddrs(3) failed: {}",
                    std::io::Error::last_os_error()
                )));
            }
            Ok(IfAddrList { ptr })
        }
    }
}

impl Drop for IfAddrList {
    fn drop(&mut self) {
        unsafe {
            freeifaddrs(self.ptr);
        }
    }
}

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
/// .find(|(name, ipaddr)| *name == "en0" && matches!(ipaddr, IpAddr::V4(_))) {
///     // This is your local IP address: 192.168.1.111
///     println!("This is your local IP address: {:?}", ipaddr);
/// }
/// ```
pub fn list_afinet_netifas() -> Result<Vec<(String, IpAddr)>, Error> {
    unsafe {
        let ptr_ifaddrs = IfAddrList::new()?;

        let mut interfaces: Vec<(String, IpAddr)> = Vec::new();
        let mut ifa = ptr_ifaddrs.ptr as *const ifaddrs;

        // An instance of `ifaddrs` is build on top of a linked list where
        // `ifaddrs.ifa_next` represent the next node in the list.
        //
        // To find the relevant interface address walk over the nodes of the
        // linked list looking for interface address which belong to the socket
        // address families AF_INET (IPv4) and AF_INET6 (IPv6)
        loop {
            let ifa_addr = (*ifa).ifa_addr;

            // If a tun device is present, no link address is assigned to it and `ifa_addr` is null.
            // See https://github.com/EstebanBorai/local-ip-address/issues/24
            if !ifa_addr.is_null() {
                match (*ifa_addr).sa_family as i32 {
                    // AF_INET IPv4 protocol implementation
                    AF_INET => {
                        let interface_address = ifa_addr;
                        let socket_addr_v4: *mut sockaddr_in =
                            interface_address as *mut sockaddr_in;
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
                    }
                    // AF_INET6 IPv6 protocol implementation
                    AF_INET6 => {
                        let interface_address = ifa_addr;
                        let socket_addr_v6: *mut sockaddr_in6 =
                            interface_address as *mut sockaddr_in6;
                        let in6_addr = (*socket_addr_v6).sin6_addr;
                        let ip_addr = Ipv6Addr::from(in6_addr.s6_addr);
                        let name = get_ifa_name(ifa)?;

                        interfaces.push((name, IpAddr::V6(ip_addr)));
                    }
                    _ => {}
                }
            }

            if (*ifa).ifa_next.is_null() {
                break;
            }

            ifa = (*ifa).ifa_next;
        }

        Ok(interfaces)
    }
}

/// Retrieves the name of a interface address
fn get_ifa_name(ifa: *const ifaddrs) -> Result<String, Error> {
    let str = unsafe { (*ifa).ifa_name as *const libc::c_char };
    let len = unsafe { strlen(str) };
    let slice = unsafe { std::slice::from_raw_parts(str as *const u8, len) };

    match String::from_utf8(slice.to_vec()) {
        Ok(s) => Ok(s),
        Err(e) => Err(Error::StrategyError(format!(
            "Failed to retrieve interface name. The name is not a valid UTF-8 string. {}",
            e
        ))),
    }
}
