<div>
  <h1 align="center">local-ip-address</h1>
  <h4 align="center">
    Retrieve system's local IP address on Rust applications using `getifaddrs`
    on Unix based systems and Win32's `GetAdaptersAddresses` on Windows
  </h4>
</div>

<div align="center">

  [![Crates.io](https://img.shields.io/crates/v/local-ip-address.svg)](https://crates.io/crates/local-ip-address)
  [![Documentation](https://docs.rs/local-ip-address/badge.svg)](https://docs.rs/local-ip-address)
  ![Build](https://github.com/EstebanBorai/local-ip-address/workflows/build/badge.svg)
  ![Clippy](https://github.com/EstebanBorai/local-ip-address/workflows/clippy/badge.svg)
  ![Formatter](https://github.com/EstebanBorai/local-ip-address/workflows/fmt/badge.svg)

</div>

A wrapper on `getifaddrs` for Unix based systems and `GetAdaptersAddresses` on
Windows which retrieves host's network interfaces.

Handy functions are provided such as `local_ip` which retrieve the local IP
address based on the host system

Is important to note that `local_ip` attempts to find commonly used network
interface names on most of the systems. As of now only support for macOS and
Windows is granted.

Network interface names may change on different Linux distribution and hardware
units. If your solution runs on different platforms its recommended to consume
the `find_af_inet` function and search for the expected interface name instead
of using `local_ip` directly.

Help improve the support for multiple systems on this crate by opening a pull
request or issue on [GitHub](https://github.com/EstebanBorai/local-ip-address).

```rust
use std::net::IpAddr;
use local_ip_address::local_ip;

assert!(matches!(local_ip().unwrap(), IpAddr));
```

You are able to iterate over a vector of tuples where the first element of
the tuple is the name of the network interface and the second is the IP
address.

```rust
use std::net::IpAddr;
use local_ip_address::find_af_inet;

let ifas = find_af_inet().unwrap();

if let Some((_, ipaddr)) = ifas
.iter()
.find(|(name, ipaddr)| *name == "en0" && matches!(ipaddr, IpAddr::V4(_))) {
    println!("This is your local IP address: {:?}", ipaddr);
    // This is your local IP address: 192.168.1.111
    assert!(matches!(ipaddr, IpAddr));
}
```

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
