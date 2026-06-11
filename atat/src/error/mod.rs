mod cme_error;
mod cms_error;
mod connection_error;

pub use cme_error::CmeError;
pub use cms_error::CmsError;
pub use connection_error::ConnectionError;
use thiserror::Error;

pub type InternalError<'a> = Error<&'a [u8]>;

/// Errors returned by the crate
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<Custom = NoCustomError> {
    /// Serial read error
    #[error("Serial read error")]
    Read,
    /// Serial write error
    #[error("Serial write error")]
    Write,
    /// Timed out while waiting for a response
    #[error("Timed out while waiting for a response")]
    Timeout,
    /// Invalid response from module
    #[error("Invalid response from module")]
    InvalidResponse,
    /// Command was aborted
    #[error("Command was aborted")]
    Aborted,
    /// Failed to parse received response
    #[error("Failed to parse received response")]
    Parse,
    /// Generic error response without any error message
    #[error("Generic error response")]
    Error,
    /// GSM Equipment related error
    #[error("GSM Equipment related error")]
    CmeError(CmeError),
    /// GSM Network related error
    #[error("GSM Network related error")]
    CmsError(CmsError),
    /// Connection Error
    #[error("Connection Error")]
    ConnectionError(ConnectionError),
    /// Error response containing any error message
    #[error("Error response containing any error message {0:?}")]
    Custom(Custom),
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            Self::Timeout => embedded_io::ErrorKind::TimedOut,
            Self::InvalidResponse => embedded_io::ErrorKind::InvalidData,
            Self::Aborted => embedded_io::ErrorKind::ConnectionAborted,
            Self::Parse => embedded_io::ErrorKind::InvalidData,
            Self::ConnectionError(e) => match e {
                ConnectionError::Unknown => embedded_io::ErrorKind::NotConnected,
                ConnectionError::NoCarrier => embedded_io::ErrorKind::ConnectionReset,
                ConnectionError::NoDialtone => embedded_io::ErrorKind::NotConnected,
                ConnectionError::Busy => embedded_io::ErrorKind::Other,
                ConnectionError::NoAnswer => embedded_io::ErrorKind::TimedOut,
            },
            _ => embedded_io::ErrorKind::Other,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub struct NoCustomError;

impl From<&[u8]> for NoCustomError {
    fn from(_: &[u8]) -> Self {
        Self
    }
}

impl core::fmt::Display for NoCustomError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}

impl<'a> InternalError<'a> {
    pub fn parse<C: From<&'a [u8]>>(self) -> Error<C> {
        match self {
            Self::Read => Error::Read,
            Self::Write => Error::Write,
            Self::Timeout => Error::Timeout,
            Self::InvalidResponse => Error::InvalidResponse,
            Self::Aborted => Error::Aborted,
            Self::Parse => Error::Parse,
            Self::Error => Error::Error,
            Self::CmeError(e) => Error::CmeError(e),
            Self::CmsError(e) => Error::CmsError(e),
            Self::ConnectionError(e) => Error::ConnectionError(e),
            Self::Custom(err) => Error::Custom(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_io::{Error as _, ErrorKind};

    #[test]
    fn test_error_kind_mapping() {
        assert_eq!(Error::Read.kind(), ErrorKind::Other);
        assert_eq!(Error::Write.kind(), ErrorKind::Other);
        assert_eq!(Error::Timeout.kind(), ErrorKind::TimedOut);
        assert_eq!(Error::InvalidResponse.kind(), ErrorKind::InvalidData);
        assert_eq!(Error::Aborted.kind(), ErrorKind::ConnectionAborted);
        assert_eq!(Error::Parse.kind(), ErrorKind::InvalidData);
        assert_eq!(Error::Error.kind(), ErrorKind::Other);
        assert_eq!(
            Error::ConnectionError(ConnectionError::Unknown).kind(),
            ErrorKind::NotConnected
        );
        assert_eq!(
            Error::ConnectionError(ConnectionError::NoCarrier).kind(),
            ErrorKind::ConnectionReset
        );
        assert_eq!(
            Error::ConnectionError(ConnectionError::NoDialtone).kind(),
            ErrorKind::NotConnected
        );
        assert_eq!(
            Error::ConnectionError(ConnectionError::Busy).kind(),
            ErrorKind::Other
        );
        assert_eq!(
            Error::ConnectionError(ConnectionError::NoAnswer).kind(),
            ErrorKind::TimedOut
        );
        assert_eq!(Error::Custom(NoCustomError).kind(), ErrorKind::Other);
    }
}
