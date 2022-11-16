<div>
  <h1 align="center">local-ip-address</h1>
  <h4 align="center">
    Retrieve system's local IP address and Network Interfaces/Adapters on
	Linux, Windows, and macOS (and other BSD-based systems).
  </h4>
</div>

<div align="center">

  [![Crates.io](https://img.shields.io/crates/v/local-ip-address.svg)](https://crates.io/crates/local-ip-address)
  [![Documentation](https://docs.rs/local-ip-address/badge.svg)](https://docs.rs/local-ip-address)
  ![Build](https://github.com/EstebanBorai/local-ip-address/workflows/build/badge.svg)
  ![Clippy](https://github.com/EstebanBorai/local-ip-address/workflows/clippy/badge.svg)
  ![Formatter](https://github.com/EstebanBorai/local-ip-address/workflows/fmt/badge.svg)

</div>

## Usage

Get the local IP address of your system by executing the `local_ip` function:

```rust
use local_ip_address::local_ip;

fn main() {
    let my_local_ip = local_ip().unwrap();

    println!("This is my local IP address: {:?}", my_local_ip);
}
```

Retrieve all the available network interfaces from both, the `AF_INET` and
the `AF_INET6` family by executing the `list_afinet_netifas` function:

```rust
use local_ip_address::list_afinet_netifas;

fn main() {
    let network_interfaces = list_afinet_netifas().unwrap();

    for (name, ip) in network_interfaces.iter() {
        println!("{}:\t{:?}", name, ip);
    }
}
```

Underlying approach on retrieving network interfaces or the local IP address
may differ based on the running operative system.

OS | Approach
--- | ---
Linux | Establishes a Netlink socket interchange to retrieve network interfaces
BSD-based | Uses of `getifaddrs` to retrieve network interfaces
Windows | Consumes Win32 API's to retrieve the network adapters table

## Operating System Support

Current supported platforms include:
  - Linux (requires at least [v0.1.0](https://github.com/EstebanBorai/local-ip-address/releases/tag/v0.1.0));
  - macOS (requires at least [v0.1.0](https://github.com/EstebanBorai/local-ip-address/releases/tag/v0.1.0));
  - Windows (requires at least [v0.3.0](https://github.com/EstebanBorai/local-ip-address/releases/tag/v0.3.0));
  - Other BSD-based (requires at least [v0.5.0](https://github.com/EstebanBorai/local-ip-address/releases/tag/v0.5.0)); including:
    - FreeBSD
    - OpenBSD
    - NetBSD
    - DragonFly

Please note that we only test the BSD implementation of this on macOS and FreeBSD, under the assumption that other BSD-based systems will behave similarly.  If you have any complications using this library on the other BSD-based, please create an [issue](https://github.com/EstebanBorai/local-ip-address/issues).

## Release

In order to create a release you must push a Git tag as follows

```sh
git tag -a <version> -m <message>
```

**Example**

```sh
git tag -a v0.1.0 -m "First release"
```

> Tags must follow semver conventions
> Tags must be prefixed with a lowercase `v` letter.

Then push tags as follows:

```sh
git push origin main --follow-tags
```

## Contributing

Every contribution to this project is welcome. Feel free to open a pull request,
an issue or just by starting this project.

## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0)
