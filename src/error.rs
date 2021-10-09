#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown error")]
    Unknown,
}
