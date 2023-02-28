use local_ip_address::{list_afinet_netifas, local_ip, local_ipv6};

fn main() {
    match local_ip() {
        Ok(ip) => println!("Local IPv4: {}", ip),
        Err(err) => println!("Failed to get local IPv4: {}", err),
    };

    match local_ipv6() {
        Ok(ip) => println!("Local IPv6: {}", ip),
        Err(err) => println!("Failed to get local IPv6: {}", err),
    };

    match list_afinet_netifas() {
        Ok(netifs) => {
            println!("Got {} interfaces", netifs.len());
            for netif in netifs {
                println!("IF: {}, IP: {}", netif.0, netif.1);
            }
        }
        Err(err) => println!("Failed to get list of network interfaces: {}", err),
    };
}
