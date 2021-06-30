## Unreleased

<Empty>

<a name="v0.4.0"></a>
## v0.4.0 (2021-06-29)

> Requires Rust: rustc 1.52.1 (9bc8c42bb 2021-05-09)

#### Features

* Provide a Netlink based implementation for Linux

* Rename methods to achieve a intuitive and easy to understand API
  * `find_af_inet` to `find_af_inet`

* Normalize errors through systems on calls to `local_ip`

<a name="v0.3.0"></a>
## v0.3.0 (2021-06-18)

> Requires Rust: rustc 1.52.1 (9bc8c42bb 2021-05-09)

#### Features

* Provide support for Windows. Kudos to @nicguzzo.

<a name="v0.2.0"></a>
## v0.2.0 (2021-06-16)

> Requires Rust: rustc 1.52.1 (9bc8c42bb 2021-05-09)

#### Features

* Implement `find_af_inet` which retrieves network interfaces which
belongs to either of both AF_INET or AF_INET6

* Implement `find_ifa` which find a network interface with a specific
name in a vector of network interfaces

#### Fixes

* Reimplement approach using `getifaddrs` instead of relying on stdout
from command execution

* Use different network interface names for "macos" and "linux" instead
of relying on "en0" for all os from the "unix" family

<a name="v0.2.0"></a>
## v0.1.0 (2021-06-15)

> Requires Rust: rustc 1.52.1 (9bc8c42bb 2021-05-09)

#### Features

Features

* Implement local IP address retrieving for Unix systems
