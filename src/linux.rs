use std::collections::HashMap;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use neli::attr::Attribute;
use neli::consts::nl::{NlmF, NlmFFlags};
use neli::consts::socket::NlFamily;
use neli::consts::rtnl::{
    Ifa, IfaFFlags, RtAddrFamily, RtScope, Rtm, RtTable, Rtprot, Rtn, RtmFFlags, RtmF, Rta, Ifla,
};
use neli::nl::{NlPayload, Nlmsghdr};
use neli::rtnl::{Ifaddrmsg, Ifinfomsg, Rtattr, Rtmsg};

use neli::socket::NlSocketHandle;
use neli::types::RtBuffer;
use neli::consts::rtnl::RtAddrFamily::{Inet, Inet6};

use crate::Error;

/// Retrieves the local IP address for this system
pub fn local_ip() -> Result<IpAddr, Error> {
    let mut netlink_socket = NlSocketHandle::connect(NlFamily::Route, None, &[])
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    let dstip = Ipv4Addr::new(192, 0, 2, 0); // reserved external IP
    let raw_dstip = u32::from(dstip).to_be();
    let route_attr = Rtattr::new(None, Rta::Dst, raw_dstip)
        .map_err(|err| Error::StrategyError(err.to_string()))?;
    let mut route_payload = RtBuffer::new();
    route_payload.push(route_attr);
    let ifroutemsg = Rtmsg {
        rtm_family: RtAddrFamily::Inet,
        rtm_dst_len: 0,
        rtm_src_len: 0,
        rtm_tos: 0,
        rtm_table: RtTable::Unspec,
        rtm_protocol: Rtprot::Unspec,
        rtm_scope: RtScope::Universe,
        rtm_type: Rtn::Unspec,
        rtm_flags: RtmFFlags::new(&[RtmF::LookupTable]),
        rtattrs: route_payload,
    };
    let netlink_message = Nlmsghdr::new(
        None,
        Rtm::Getroute,
        NlmFFlags::new(&[NlmF::Request]),
        None,
        None,
        NlPayload::Payload(ifroutemsg),
    );

    netlink_socket
        .send(netlink_message)
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    for response in netlink_socket.iter(false) {
        let header: Nlmsghdr<Rtm, Rtmsg> = response.map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred retrieving Netlink's socket response",
            ))
        })?;

        if let NlPayload::Empty = header.nl_payload {
            continue;
        }

        if header.nl_type != Rtm::Newroute {
            return Err(Error::StrategyError(String::from(
                "The Netlink header type is not the expected",
            )));
        }

        let p = header.get_payload().map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred getting Netlink's header payload",
            ))
        })?;

        if p.rtm_scope != RtScope::Universe {
            continue;
        }

        for rtattr in p.rtattrs.iter() {
            if rtattr.rta_type == Rta::Prefsrc {
                let addr = Ipv4Addr::from(u32::from_be(rtattr.get_payload_as::<u32>().map_err(
                    |_| {
                        Error::StrategyError(String::from(
                            "An error occurred retrieving Netlink's route payload attribute",
                        ))
                    },
                )?));
                return Ok(IpAddr::V4(addr));
            }
        }
    }

    let ifaddrmsg = Ifaddrmsg {
        ifa_family: RtAddrFamily::Inet,
        ifa_prefixlen: 0,
        ifa_flags: IfaFFlags::empty(),
        ifa_scope: 0,
        ifa_index: 0,
        rtattrs: RtBuffer::new(),
    };
    let netlink_message = Nlmsghdr::new(
        None,
        Rtm::Getaddr,
        NlmFFlags::new(&[NlmF::Request, NlmF::Root]),
        None,
        None,
        NlPayload::Payload(ifaddrmsg),
    );

    netlink_socket
        .send(netlink_message)
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    for response in netlink_socket.iter(false) {
        let header: Nlmsghdr<Rtm, Ifaddrmsg> = response.map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred retrieving Netlink's socket response",
            ))
        })?;

        if let NlPayload::Empty = header.nl_payload {
            continue;
        }

        if header.nl_type != Rtm::Newaddr {
            return Err(Error::StrategyError(String::from(
                "The Netlink header type is not the expected",
            )));
        }

        let p = header.get_payload().map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred getting Netlink's header payload",
            ))
        })?;

        if RtScope::from(p.ifa_scope) != RtScope::Universe {
            continue;
        }

        for rtattr in p.rtattrs.iter() {
            if rtattr.rta_type == Ifa::Local {
                let addr = Ipv4Addr::from(u32::from_be(rtattr.get_payload_as::<u32>().map_err(
                    |_| {
                        Error::StrategyError(String::from(
                            "An error occurred retrieving Netlink's route payload attribute",
                        ))
                    },
                )?));
                return Ok(IpAddr::V4(addr));
            }
        }
    }

    Err(Error::LocalIpAddressNotFound)
}

/// Perform a search over the system's network interfaces using Netlink Route information,
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
    let mut netlink_socket = NlSocketHandle::connect(NlFamily::Route, None, &[])
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    // First get list of interfaces via RTM_GETLINK

    let ifroutemsg = Rtmsg {
        rtm_family: RtAddrFamily::Unspecified,
        rtm_dst_len: 0,
        rtm_src_len: 0,
        rtm_tos: 0,
        rtm_table: RtTable::Unspec,
        rtm_protocol: Rtprot::Unspec,
        rtm_scope: RtScope::Universe,
        rtm_type: Rtn::Unspec,
        rtm_flags: RtmFFlags::new(&[RtmF::LookupTable]),
        rtattrs: RtBuffer::new(),
    };
    let netlink_message = Nlmsghdr::new(
        None,
        Rtm::Getlink,
        NlmFFlags::new(&[NlmF::Request, NlmF::Dump]),
        None,
        None,
        NlPayload::Payload(ifroutemsg),
    );

    netlink_socket
        .send(netlink_message)
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    let mut if_indexes = HashMap::new();

    for response in netlink_socket.iter(false) {
        let header: Nlmsghdr<Rtm, Ifinfomsg> = response.map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred retrieving Netlink's socket response",
            ))
        })?;

        if let NlPayload::Empty = header.nl_payload {
            continue;
        }

        if header.nl_type != Rtm::Newlink {
            return Err(Error::StrategyError(String::from(
                "The Netlink header type is not the expected",
            )));
        }

        let p = header.get_payload().map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred getting Netlink's header payload",
            ))
        })?;

        for rtattr in p.rtattrs.iter() {
            if rtattr.rta_type == Ifla::Ifname {
                let ifname = parse_ifname(rtattr.payload().as_ref())?;
                if_indexes.insert(p.ifi_index, ifname);
                break;
            }
        }
    }

    // Secondly get addresses of interfaces via RTM_GETADDR

    let ifroutemsg = Rtmsg {
        rtm_family: RtAddrFamily::Unspecified,
        rtm_dst_len: 0,
        rtm_src_len: 0,
        rtm_tos: 0,
        rtm_table: RtTable::Unspec,
        rtm_protocol: Rtprot::Unspec,
        rtm_scope: RtScope::Universe,
        rtm_type: Rtn::Unspec,
        rtm_flags: RtmFFlags::new(&[RtmF::LookupTable]),
        rtattrs: RtBuffer::new(),
    };
    let netlink_message = Nlmsghdr::new(
        None,
        Rtm::Getaddr,
        NlmFFlags::new(&[NlmF::Request, NlmF::Dump]),
        None,
        None,
        NlPayload::Payload(ifroutemsg),
    );

    netlink_socket
        .send(netlink_message)
        .map_err(|err| Error::StrategyError(err.to_string()))?;

    let mut interfaces = Vec::new();

    for response in netlink_socket.iter(false) {
        let header: Nlmsghdr<Rtm, Ifaddrmsg> = response.map_err(|err| {
            Error::StrategyError(format!(
                "An error occurred retrieving Netlink's socket response: {err}"
            ))
        })?;

        if let NlPayload::Empty = header.nl_payload {
            continue;
        }

        if header.nl_type != Rtm::Newaddr {
            return Err(Error::StrategyError(String::from(
                "The Netlink header type is not the expected",
            )));
        }

        let p = header.get_payload().map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred getting Netlink's header payload",
            ))
        })?;

        if p.ifa_family != Inet6 && p.ifa_family != Inet {
            Err(Error::StrategyError(format!(
                "Netlink payload has unsupported family: {:?}",
                p.ifa_family
            )))?
        }

        let mut ipaddr = None;

        for rtattr in p.rtattrs.iter() {
            if rtattr.rta_type == Ifa::Label {
                let ifname = parse_ifname(rtattr.payload().as_ref())?;
                if_indexes.insert(p.ifa_index, ifname);
            } else if rtattr.rta_type == Ifa::Address {
                if ipaddr.is_some() {
                    // do not override IFA_LOCAL
                    continue;
                }
                if p.ifa_family == Inet6 {
                    let rtaddr = Ipv6Addr::from(u128::from_be(
                        rtattr.get_payload_as::<u128>().map_err(|_| {
                            Error::StrategyError(String::from(
                                "An error occurred retrieving Netlink's route payload attribute",
                            ))
                        })?,
                    ));
                    ipaddr = Some(IpAddr::V6(rtaddr));
                } else {
                    let rtaddr = Ipv4Addr::from(u32::from_be(
                        rtattr.get_payload_as::<u32>().map_err(|_| {
                            Error::StrategyError(String::from(
                                "An error occurred retrieving Netlink's route payload attribute",
                            ))
                        })?,
                    ));
                    ipaddr = Some(IpAddr::V4(rtaddr));
                }
            } else if rtattr.rta_type == Ifa::Local {
                if p.ifa_family == Inet6 {
                    let rtlocal = Ipv6Addr::from(u128::from_be(
                        rtattr.get_payload_as::<u128>().map_err(|_| {
                            Error::StrategyError(String::from(
                                "An error occurred retrieving Netlink's route payload attribute",
                            ))
                        })?,
                    ));
                    ipaddr = Some(IpAddr::V6(rtlocal));
                } else {
                    let rtlocal = Ipv4Addr::from(u32::from_be(
                        rtattr.get_payload_as::<u32>().map_err(|_| {
                            Error::StrategyError(String::from(
                                "An error occurred retrieving Netlink's route payload attribute",
                            ))
                        })?,
                    ));
                    ipaddr = Some(IpAddr::V4(rtlocal));
                }
            }
        }

        if let Some(ipaddr) = ipaddr {
            if let Some(ifname) = if_indexes.get(&p.ifa_index) {
                interfaces.push((ifname.clone(), ipaddr));
            }
        }
    }

    Ok(interfaces)
}

fn parse_ifname(bytes: &[u8]) -> Result<String, Error> {
    let ifname = CStr::from_bytes_with_nul(bytes)
        .map_err(|_| {
            Error::StrategyError(String::from(
                "An error occurred convert interface name to string",
            ))
        })?
        .to_string_lossy()
        .to_string();
    Ok(ifname)
}
