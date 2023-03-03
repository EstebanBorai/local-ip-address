use std::{
    alloc::{alloc, dealloc, Layout},
    net::IpAddr,
    ptr::{NonNull, self},
    slice,
    marker::PhantomData,
    ops::Deref,
    mem,
};

use windows_sys::Win32::{
    Foundation::{
        GetLastError, BOOL, ERROR_ADDRESS_NOT_ASSOCIATED, ERROR_BUFFER_OVERFLOW,
        ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY,
        ERROR_NOT_SUPPORTED, ERROR_NO_DATA, ERROR_SUCCESS, WIN32_ERROR,
    },
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GetIpForwardTable, GET_ADAPTERS_ADDRESSES_FLAGS,
        IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_UNICAST_ADDRESS_LH, MIB_IPFORWARDTABLE,
    },
    Networking::WinSock::{
        ADDRESS_FAMILY, AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6, SOCKADDR,
    },
    System::{
        Diagnostics::Debug::{
            FormatMessageW, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
        },
        Memory::LocalFree,
    },
};

use crate::error::Error;

/// Retrieves the local ip addresses for this system.
pub(crate) fn list_local_ip_addresses(family: ADDRESS_FAMILY) -> Result<Vec<IpAddr>, Error> {
    /// An IPv4 address of 0.0.0.0 in the dwForwardDest member of the MIB_IPFORWARDROW structure is considered a
    /// default route.
    const DEFAULT_ROUTE: u32 = 0;

    // There can be multiple default routes (e.g. wifi and ethernet).
    let default_route_interface_indices: Vec<u32> = {
        let ip_forward_table = get_ip_forward_table(0).map_err(|error| match error {
            ERROR_NO_DATA | ERROR_NOT_SUPPORTED => Error::LocalIpAddressNotFound,
            error_code => Error::StrategyError(format_error_code(error_code)),
        })?;

        let table = unsafe {
            slice::from_raw_parts(
                ip_forward_table.table.as_ptr(),
                ip_forward_table.dwNumEntries.try_into().unwrap(),
            )
        };

        table
            .iter()
            .filter_map(|row| {
                if row.dwForwardDest == DEFAULT_ROUTE {
                    Some(row.dwForwardIfIndex)
                } else {
                    None
                }
            })
            .collect()
    };

    let adapter_addresses = get_adapter_addresses(family, 0).map_err(|error| match error {
        ERROR_ADDRESS_NOT_ASSOCIATED | ERROR_NO_DATA => Error::LocalIpAddressNotFound,
        error_code => Error::StrategyError(format_error_code(error_code)),
    })?;
    let adapter_addresses_iter = LinkedListIter::new(Some(adapter_addresses.ptr));

    let local_ip_address = adapter_addresses_iter
        .filter(|adapter_address| {
            let interface_index = unsafe { adapter_address.Anonymous1.Anonymous.IfIndex };
            default_route_interface_indices.contains(&interface_index)
        })
        .flat_map(|default_adapter_address| {
            let unicast_addresses_iter =
                LinkedListIter::new(NonNull::new(default_adapter_address.FirstUnicastAddress));

            unicast_addresses_iter.filter_map(|unicast_address| {
                let socket_address = NonNull::new(unicast_address.Address.lpSockaddr)?;
                get_ip_address_from_socket_address(socket_address)
            })
        })
        .collect();

    Ok(local_ip_address)
}

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
    let adapter_addresses = get_adapter_addresses(AF_UNSPEC, 0)
        .map_err(|error_code| Error::StrategyError(format_error_code(error_code)))?;
    let adapter_addresses_iter = LinkedListIter::new(Some(adapter_addresses.ptr));

    let network_interfaces = adapter_addresses_iter
        .flat_map(|adapter_address| {
            let unicast_addresses_iter =
                LinkedListIter::new(NonNull::new(adapter_address.FirstUnicastAddress));

            let friendly_name = unsafe {
                #[allow(unused_unsafe)]
                // SAFETY: This is basically how `wcslen` works under the hood. `wcslen` is unsafe because the pointer
                // is not checked for null and if there is no null-terminating character, it will run forever.
                // Therefore, safety relies on the operating sysytem always returning a valid string.
                let len = unsafe {
                    let mut ptr = adapter_address.FriendlyName;
                    while *ptr != 0 {
                        ptr = ptr.offset(1);
                    }
                    ptr.offset_from(adapter_address.FriendlyName)
                        .try_into()
                        .unwrap()
                };

                slice::from_raw_parts(adapter_address.FriendlyName, len)
            };

            unicast_addresses_iter.filter_map(|unicast_address| {
                let socket_address = NonNull::new(unicast_address.Address.lpSockaddr)?;
                get_ip_address_from_socket_address(socket_address)
                    .map(|ip_address| (String::from_utf16_lossy(friendly_name), ip_address))
            })
        })
        .collect();

    Ok(network_interfaces)
}

/// The [GetIpForwardTable][GetIpForwardTable] function retrieves the IPv4 routing table.
///
/// [GetIpForwardTable]: https://docs.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getipforwardtable
fn get_ip_forward_table(order: BOOL) -> Result<ReadonlyResource<MIB_IPFORWARDTABLE>, WIN32_ERROR> {
    // The minimum size of a routing table.
    const INITIAL_BUFFER_SIZE: u32 = mem::size_of::<MIB_IPFORWARDTABLE>() as u32;

    let mut size = INITIAL_BUFFER_SIZE;

    loop {
        let ip_forward_table =
            ReadonlyResource::new(size.try_into().unwrap()).ok_or(ERROR_NOT_ENOUGH_MEMORY)?;

        let result = unsafe { GetIpForwardTable(ip_forward_table.ptr.as_ptr(), &mut size, order) };

        break match result {
            ERROR_SUCCESS => Ok(ip_forward_table),
            ERROR_INSUFFICIENT_BUFFER => continue,
            #[cfg(debug_assertions)]
            ERROR_INVALID_PARAMETER => unreachable!(),
            ERROR_NO_DATA => Err(ERROR_NO_DATA),
            ERROR_NOT_SUPPORTED => Err(ERROR_NOT_SUPPORTED),
            error => Err(error),
        };
    }
}

/// The [GetAdaptersAddresses][GetAdaptersAddresses] function retrieves the addresses associated with the adapters on
/// the local computer.
///
/// [GetAdaptersAddresses]: https://docs.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getadaptersaddresses
fn get_adapter_addresses(
    family: ADDRESS_FAMILY,
    flags: GET_ADAPTERS_ADDRESSES_FLAGS,
) -> Result<ReadonlyResource<IP_ADAPTER_ADDRESSES_LH>, WIN32_ERROR> {
    // The recommended buffer size is 15kb.
    const INITIAL_BUFFER_SIZE: u32 = 15000;

    let mut size: u32 = INITIAL_BUFFER_SIZE;

    loop {
        let adapter_addresses =
            ReadonlyResource::new(size.try_into().unwrap()).ok_or(ERROR_NOT_ENOUGH_MEMORY)?;

        let result = unsafe {
            GetAdaptersAddresses(
                family as u32,
                flags,
                ptr::null_mut(),
                adapter_addresses.ptr.as_ptr(),
                &mut size,
            )
        };

        break match result {
            ERROR_SUCCESS => Ok(adapter_addresses),
            ERROR_BUFFER_OVERFLOW => continue,
            #[cfg(debug_assertions)]
            ERROR_INVALID_PARAMETER => unreachable!(),
            ERROR_ADDRESS_NOT_ASSOCIATED => Err(ERROR_ADDRESS_NOT_ASSOCIATED),
            ERROR_NOT_ENOUGH_MEMORY => Err(ERROR_NOT_ENOUGH_MEMORY),
            ERROR_NO_DATA => Err(ERROR_NO_DATA),
            error => Err(error),
        };
    }
}

/// Converts a Windows socket address to an ip address.
fn get_ip_address_from_socket_address(socket_address: NonNull<SOCKADDR>) -> Option<IpAddr> {
    let socket_address_family = u32::from(unsafe { socket_address.as_ref().sa_family }) as u16;

    if socket_address_family == AF_INET {
        let socket_address = unsafe { socket_address.cast::<SOCKADDR_IN>().as_ref() };
        let address = unsafe { socket_address.sin_addr.S_un.S_addr };
        let ipv4_address = IpAddr::from(address.to_ne_bytes());
        Some(ipv4_address)
    } else if socket_address_family == AF_INET6 {
        let socket_address = unsafe { socket_address.cast::<SOCKADDR_IN6>().as_ref() };
        let address = unsafe { socket_address.sin6_addr.u.Byte };
        let ipv6_address = IpAddr::from(address);
        Some(ipv6_address)
    } else {
        None
    }
}

/// Formats a Windows API error code to a localized error message.
// Based on the example in https://docs.microsoft.com/en-us/globalization/localizability/win32-formatmessage.
fn format_error_code(error_code: WIN32_ERROR) -> String {
    let mut wide_ptr = ptr::null_mut::<u16>();

    let len = unsafe {
        FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            ptr::null(),
            error_code,
            0,
            &mut wide_ptr as *mut _ as *mut _,
            0,
            ptr::null(),
        )
    };

    debug_assert!(
        len > 0,
        "Retrieving static error message from the OS for error code {} failed with error code {}.",
        error_code,
        unsafe { GetLastError() }
    );

    let slice = unsafe { slice::from_raw_parts(wide_ptr, len.try_into().unwrap()) };
    let error_message = String::from_utf16_lossy(slice);

    unsafe {
        LocalFree(wide_ptr as isize);
    }

    error_message
}

/// Wrapper type around a pointer to a Windows API structure.
///
/// This type ensures that the memory allocated is freed automatically and fields are not overwritten.
struct ReadonlyResource<T> {
    ptr: NonNull<T>,
    layout: Layout,
}

/// A trait to allow low level linked list data structures to be used as Rust iterators.
///
/// The networking data structures often contain linked lists, which (unfortunately) are a separate types with
/// differently named next fields. This trait aims to abstract over these differences and offers helper structs to
/// transform linked lists into iterators.
trait LinkedListIterator {
    /// Returns the pointer to the next value.
    fn next(&self) -> Option<NonNull<Self>>;
}

/// Adaptor to convert a linked list to an iterator of references.
struct LinkedListIter<'linked_list, T: LinkedListIterator> {
    node: Option<NonNull<T>>,
    __phantom_lifetime: PhantomData<&'linked_list T>,
}

impl<T> ReadonlyResource<T> {
    fn new(size: usize) -> Option<ReadonlyResource<T>> {
        let layout = Layout::from_size_align(size, mem::align_of::<T>()).ok()?;
        let ptr = NonNull::new(unsafe { alloc(layout).cast() })?;
        Some(ReadonlyResource { ptr, layout })
    }
}

impl<T> Deref for ReadonlyResource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Drop for ReadonlyResource<T> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr().cast(), self.layout);
        }
    }
}

impl LinkedListIterator for IP_ADAPTER_ADDRESSES_LH {
    fn next(&self) -> Option<NonNull<Self>> {
        NonNull::new(self.Next)
    }
}

impl LinkedListIterator for IP_ADAPTER_UNICAST_ADDRESS_LH {
    fn next(&self) -> Option<NonNull<Self>> {
        NonNull::new(self.Next)
    }
}

impl<'linked_list, T: LinkedListIterator> LinkedListIter<'linked_list, T> {
    /// Creates a new [LinkedListIter] from a pointer to the head of the linked list.
    pub fn new(head: Option<NonNull<T>>) -> Self {
        Self {
            node: head,
            __phantom_lifetime: PhantomData,
        }
    }
}

impl<'linked_list, T: LinkedListIterator> Iterator for LinkedListIter<'linked_list, T> {
    type Item = &'linked_list T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.node?;
        let item = unsafe { node.as_ref() };
        self.node = item.next();
        Some(item)
    }
}
