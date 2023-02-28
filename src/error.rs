#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Returned when `local_ip` is unable to find the system's local IP address
    /// in the collection of network interfaces
    #[error("The Local IP Address wasn't available in the network interfaces list/table")]
    LocalIpAddressNotFound,
    /// Returned when an error occurs in the strategy level.
    /// The error message may include any internal strategy error if available
    #[error("An error occurred executing the underlying strategy error.\n{0}")]
    StrategyError(String),
    /// Returned when the current platform is not yet supported
    #[error("The current platform: `{0}`, is not supported")]
    PlatformNotSupported(String),
}
