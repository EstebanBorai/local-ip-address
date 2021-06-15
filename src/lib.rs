/*!
Provides utility functions to get system's local network IP address by executing
OS commands in the host machine.

The output from the executed command is then parsed and an instance of a `IpAddr`
is returned from the function.

```ignore
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
*/
use regex::Regex;
use std::env;
use std::net::{AddrParseError, IpAddr};
use std::process::Command;
use std::str::FromStr;
use std::string::FromUtf8Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid IP address, failed to parse `{0}` into an IP address")]
    InvalidIpAddress(String, AddrParseError),
    #[error("Invalid UTF-8 output found")]
    InvalidUtf8(FromUtf8Error),
    #[error("An error ocurred executing `{0}` on your platform (`{1}`)")]
    CommandFailed(String, String, std::io::Error),
    #[error("The current platform `{0}` is not supported.")]
    PlatformNotSupported(String),
    #[error("Failed to match IP from output. The following command output is not supported\n{0}")]
    OutputNotSupported(String),
}

pub type Result = std::result::Result<IpAddr, Error>;

/// Executes a command locally to gather system's local IP address.
///
/// Returns an `Err` with the `Error` variant `PlatformNotSupported` and the
/// name of the current platform name if theres no strategy available for this
/// platform.
pub fn local_ip() -> Result {
    if cfg!(target_family = "unix") {
        return unix();
    }

    Err(Error::PlatformNotSupported(env::consts::OS.to_string()))
}

/// Unix strategy relies on executing `ipconfig` and extracting the IP address
/// from the command output using a regular expression.
///
/// The executed command is:
///
/// ```ignore
/// ipconfig getifaddr en0
/// ```
fn unix() -> Result {
    let regex = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
    let output = Command::new("ipconfig")
        .arg("getifaddr")
        .arg("en0")
        .output()
        .map_err(|error| {
            Error::CommandFailed(String::from("ipconfig"), env::consts::OS.to_string(), error)
        })?;
    let ip_addr = String::from_utf8(output.stdout).map_err(Error::InvalidUtf8)?;

    if let Some(captures) = regex.captures(&ip_addr) {
        if let Some(matches) = captures.get(0) {
            return IpAddr::from_str(matches.as_str())
                .map_err(|error| Error::InvalidIpAddress(ip_addr.to_string(), error));
        }

        return Err(Error::OutputNotSupported(format!("{:?}", captures)));
    }

    Err(Error::OutputNotSupported(ip_addr))
}
