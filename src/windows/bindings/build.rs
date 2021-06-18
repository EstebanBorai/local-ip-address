fn main() {
    windows::build! {
        Windows::Win32::NetworkManagement::IpHelper::{GetIpAddrTable,GetInterfaceInfo,GetAdaptersAddresses},
        Windows::Win32::Networking::WinSock::{SOCKADDR_IN,SOCKADDR_IN6},
        Windows::Win32::System::Diagnostics::Debug::*
        //Windows::Win32::System::Diagnostics::Debug::{ERROR_BUFFER_OVERFLOW, ERROR_INSUFFICIENT_BUFFER, NO_ERROR}
    };
}
