use std::mem;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use neli::attr::Attribute;
use neli::consts::nl::{NlmF, NlmFFlags};
use neli::consts::socket::NlFamily;
use neli::consts::rtnl::{Ifa, IfaFFlags, RtAddrFamily, RtScope, Rtm};
use neli::nl::{NlPayload, Nlmsghdr};
use neli::rtnl::Ifaddrmsg;
use neli::socket::NlSocketHandle;
use neli::types::RtBuffer;

use crate::error::Error;

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
    let mut netlink_socket =
        NlSocketHandle::connect(NlFamily::Route, None, &[]).map_err(|_err| Error::Unknown)?;
    let ifaddrmsg = make_ifaddrmsg();
    let netlink_payload = NlPayload::Payload(ifaddrmsg);
    let netlink_message = make_netlink_message(netlink_payload);

    netlink_socket
        .send(netlink_message)
        .map_err(|_err| Error::Unknown)?;

    let mut addrs = Vec::<Ipv4Addr>::with_capacity(1);

    for response in netlink_socket.iter(false) {
        let header: Nlmsghdr<_, Ifaddrmsg> = response.map_err(|_| Error::Unknown)?;

        if let NlPayload::Empty = header.nl_payload {
            continue;
        }

        if header.nl_type != Rtm::Newaddr.into() {
            return Err(Error::Unknown);
        }

        let p = header.get_payload().map_err(|_| Error::Unknown)?;

        if RtScope::from(p.ifa_scope) != RtScope::Universe {
            continue;
        }

        for rtattr in p.rtattrs.iter() {
            if rtattr.rta_type == Ifa::Local {
                addrs.push(Ipv4Addr::from(u32::from_be(
                    rtattr.get_payload_as::<u32>().map_err(|_| Error::Unknown)?,
                )));
            }
        }
    }

    if let Some(local_ip) = addrs.first() {
        let ipaddr = IpAddr::V4(local_ip.to_owned());

        return Ok(ipaddr);
    }

    Err(Error::Unknown)
}
