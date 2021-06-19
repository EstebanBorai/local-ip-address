use libc::{wchar_t, wcslen};
use memalloc::{allocate, deallocate};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use bindings::Windows::Win32::System::Diagnostics::Debug::{ERROR_BUFFER_OVERFLOW, NO_ERROR};
use bindings::Windows::Win32::Networking::WinSock::{SOCKADDR_IN, SOCKADDR_IN6};
use bindings::Windows::Win32::NetworkManagement::IpHelper::{
    ADDRESS_FAMILY, AF_INET, AF_INET6, AF_UNSPEC, GET_ADAPTERS_ADDRESSES_FLAGS,
    IP_ADAPTER_ADDRESSES_LH, GetAdaptersAddresses,
};

use crate::Error;

/// Perform a search over the system's network interfaces using `GetAdaptersAddresses`,
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
pub fn find_af_inet() -> Result<Vec<(String, IpAddr)>, Error> {
    let mut out: Vec<(String, IpAddr)> = Vec::new();
    // 20kb should be enough to prevent realloc
    let mut dwsize: u32 = 2000;
    let mut mem = unsafe { allocate(dwsize as usize) } as *mut IP_ADAPTER_ADDRESSES_LH;
    let mut n_tries = 3;
    let mut ret_val;

    loop {
        let old_size = dwsize as usize;

        ret_val = unsafe {
            GetAdaptersAddresses(
                ADDRESS_FAMILY(AF_UNSPEC.0),
                GET_ADAPTERS_ADDRESSES_FLAGS(0x0),
                std::ptr::null_mut::<std::ffi::c_void>(),
                mem,
                &mut dwsize,
            )
        };

        if ret_val != ERROR_BUFFER_OVERFLOW.0 || n_tries <= 0 {
            break;
        }

        unsafe { deallocate(mem as *mut u8, old_size as usize) };

        mem = unsafe { allocate(dwsize as usize) as *mut IP_ADAPTER_ADDRESSES_LH };
        n_tries -= 1;
    }

    if ret_val == NO_ERROR.0 {
        let mut cur = mem;

        while !cur.is_null() {
            let fname = unsafe { (*cur).FriendlyName.0 };
            let len = unsafe { wcslen(fname as *const wchar_t) };
            let slice = unsafe { std::slice::from_raw_parts(fname, len) };

            let mut cur_a = unsafe { (*cur).FirstUnicastAddress };
            while !cur_a.is_null() {
                let addr = unsafe { (*cur_a).Address };
                let sockaddr = unsafe { *addr.lpSockaddr };
                if sockaddr.sa_family == AF_INET6.0 as u16 {
                    let sockaddr: *mut SOCKADDR_IN6 = addr.lpSockaddr as *mut SOCKADDR_IN6;
                    let a = unsafe { (*sockaddr).sin6_addr.u.Byte };
                    let ipv6 = Ipv6Addr::from(a);
                    let ip = IpAddr::V6(ipv6);
                    let name = String::from_utf16(slice).unwrap();
                    out.push((name, ip));
                } else if sockaddr.sa_family == AF_INET.0 as u16 {
                    let sockaddr: *mut SOCKADDR_IN = addr.lpSockaddr as *mut SOCKADDR_IN;
                    let a = unsafe { (*sockaddr).sin_addr.S_un.S_addr };
                    let ipv4 = if cfg!(target_endian = "little") {
                        Ipv4Addr::from(a.swap_bytes())
                    } else {
                        Ipv4Addr::from(a)
                    };

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
            deallocate(mem as *mut u8, dwsize as usize);
        }

        return Err(Error::GetAdaptersAddresses(ret_val));
    }

    unsafe {
        deallocate(mem as *mut u8, dwsize as usize);
    }

    Ok(out)
}
