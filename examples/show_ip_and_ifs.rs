use local_ip_address::{list_afinet_netifas, local_ip};

fn main() {
    match local_ip() {
        Ok(ip) => println!("Local IP: {}", ip),
        Err(err) => println!("Failed to get local IP: {}", err),
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
