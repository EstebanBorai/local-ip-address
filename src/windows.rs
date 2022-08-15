use libc::{wchar_t, wcslen, malloc, free};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6};
use windows_sys::Win32::NetworkManagement::IpHelper::{IP_ADAPTER_ADDRESSES_LH, GetAdaptersAddresses};

use crate::error::Error;

/// Perform a search over the system's network interfaces using `GetAdaptersAddresses`,
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
    const ERROR_BUFFER_OVERFLOW: u32 = 111;
    const NO_ERROR: u32 = 0;
    let mut out: Vec<(String, IpAddr)> = Vec::new();
    // 20kb should be enough to prevent realloc
    let mut dwsize: u32 = 2000;
    let mut mem = unsafe { malloc(dwsize as usize) } as *mut IP_ADAPTER_ADDRESSES_LH;
    let mut n_tries = 3;
    let mut ret_val;

    loop {
        ret_val = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC,
                0,
                std::ptr::null_mut::<std::ffi::c_void>(),
                mem,
                &mut dwsize,
            )
        };

        if ret_val != ERROR_BUFFER_OVERFLOW || n_tries <= 0 {
            break;
        }

        unsafe { free(mem as *mut _) };

        mem = unsafe { malloc(dwsize as usize) as *mut IP_ADAPTER_ADDRESSES_LH };
        n_tries -= 1;
    }

    if ret_val == NO_ERROR {
        let mut cur = mem;

        while !cur.is_null() {
            let fname = unsafe { (*cur).FriendlyName };
            let len = unsafe { wcslen(fname as *const wchar_t) };
            let slice = unsafe { std::slice::from_raw_parts(fname, len) };

            let mut cur_a = unsafe { (*cur).FirstUnicastAddress };
            while !cur_a.is_null() {
                let addr = unsafe { (*cur_a).Address };
                let sockaddr = unsafe { *addr.lpSockaddr };
                let socket_address_family = u32::from(sockaddr.sa_family);
                if socket_address_family == AF_INET6 {
                    let sockaddr: *mut SOCKADDR_IN6 = addr.lpSockaddr as *mut SOCKADDR_IN6;
                    let a = unsafe { (*sockaddr).sin6_addr.u.Byte };
                    let ipv6 = Ipv6Addr::from(a);
                    let ip = IpAddr::V6(ipv6);
                    let name = String::from_utf16(slice).unwrap();
                    out.push((name, ip));
                } else if socket_address_family == AF_INET {
                    let sockaddr: *mut SOCKADDR_IN = addr.lpSockaddr as *mut SOCKADDR_IN;
                    let a = unsafe { (*sockaddr).sin_addr.S_un.S_addr };
                    let ipv4 = Ipv4Addr::from(a);

                    let ip = IpAddr::V4(ipv4);
                    let name = String::from_utf16(slice).unwrap();

                    out.push((name, ip));
                }
                cur_a = unsafe { (*cur_a).Next };
            }

            cur = unsafe { (*cur).Next };
        }
    } else {
        unsafe {
            free(mem as *mut _);
        }

        return Err(Error::StrategyError(format!(
            "Failed to get adapter addresses. Error: {}",
            ret_val
        )));
    }

    unsafe {
        free(mem as *mut _);
    }

    Ok(out)
}
