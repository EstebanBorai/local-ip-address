In order to publish local-ip-address with windows support we rely
on using windows::include_bindings! macro.

Given that any crate published should only have dependencies which
are other crates on the same package registry (crates.io) we should
publish a separated crate for this bindings to be available.

Refer: https://github.com/microsoft/windows-rs/issues/819
Refer: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies
