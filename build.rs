fn main() {
    #[cfg(target_family = "windows")]
    windows::build! {
        Windows::Win32::{
            NetworkManagement::IpHelper::GetAdaptersAddresses,
            Networking::WinSock::{SOCKADDR_IN,SOCKADDR_IN6}
        }
    };
}
