mod target;

pub mod error;

use network_interface::NetworkInterface;

pub type Result = std::result::Result<NetworkInterface, Box<dyn std::error::Error>>;

pub fn local_ip() -> Result {
    #[cfg(target_os = "linux")]
    {
        use network_interface::NetworkInterfaceConfig;

        let netifas = NetworkInterface::show()?;

        if let Some(net_ifa) = netifas.iter().find(|net_ifa| net_ifa.name == "en0") {
            return Ok(net_ifa.to_owned());
        }

        todo!()
    }

    #[cfg(target_os = "macos")]
    {
        use network_interface::NetworkInterfaceConfig;

        let netifas = NetworkInterface::show()?;

        if let Some(net_ifa) = netifas.iter().find(|net_ifa| net_ifa.name == "en0") {
            return Ok(net_ifa.to_owned());
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::local_ip;

    #[test]
    #[cfg(target_os = "macos")]
    fn find_local_ip_macos() {
        let local_ip_netifa = local_ip().unwrap();

        assert!(!local_ip_netifa.name.is_empty());
        assert_eq!(local_ip_netifa.name, "en0");
    }
}
