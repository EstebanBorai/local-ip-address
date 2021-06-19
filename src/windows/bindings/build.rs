fn main() {
    windows::build! {
        Windows::Win32::{
            NetworkManagement::IpHelper::GetAdaptersAddresses,
            Networking::WinSock::{SOCKADDR_IN,SOCKADDR_IN6},
            System::Diagnostics::Debug::*
        }
    };
}
