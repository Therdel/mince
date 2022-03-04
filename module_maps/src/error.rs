use thiserror::Error;

/// The result type used by this library.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(target_os="windows")]
    #[error("error with enumerating the mappings")]
    MappingEnumerationError(#[from] windows::core::Error),

    #[cfg(target_os="linux")]
    #[error("error with enumerating the mappings")]
    MappingEnumerationError,
}
