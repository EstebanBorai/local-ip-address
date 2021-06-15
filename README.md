<div>
  <h1 align="center">local-ip-address</h1>
  <h4 align="center">Retrive system's local IP address</h4>
</div>

<div align="center">

  [![Crates.io](https://img.shields.io/crates/v/local-ip-address.svg)](https://crates.io/crates/local-ip-address)
  [![Documentation](https://docs.rs/local-ip-address/badge.svg)](https://docs.rs/local-ip-address)
  ![Build](https://github.com/EstebanBorai/local-ip-address/workflows/build/badge.svg)
  ![Clippy](https://github.com/EstebanBorai/local-ip-address/workflows/clippy/badge.svg)
  ![Formatter](https://github.com/EstebanBorai/local-ip-address/workflows/fmt/badge.svg)

</div>

Provides utility functions to get system's local network IP address by executing
OS commands in the host machine.

The output from the executed command is then parsed and an instance of a `IpAddr`
is returned from the function.

```rust
use local_ip_address::local_ip;

fn main() {
    let my_local_ip_address = local_ip().unwrap();

    println!("{:?}", my_local_ip_address);
}
```

Every host may or may not have a different approach on gathering the local IP
address.

`local-ip-address` crate implements [Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html#conditional-compilation)
to execute different approaches based on host machine operative system.


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
