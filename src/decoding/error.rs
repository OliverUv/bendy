use alloc::{
    format,
    str::Utf8Error,
    string::{FromUtf8Error, String, ToString},
};
use core::{fmt::Display, num::ParseIntError};

#[cfg(feature = "std")]
use std::sync::Arc;

use snafu::Snafu;

use crate::state_tracker;

#[derive(Debug, Clone, Snafu)]
pub struct Error {
    context: Option<String>,
    source: ErrorKind,
}

// An enumeration of potential errors that appear during bencode deserialization.
#[derive(Debug, Clone, Snafu)]
pub enum ErrorKind {
    /// Error that occurs if the serialized structure contains invalid semantics.
    #[cfg(feature = "std")]
    #[snafu(display("malformed content discovered: {}", source))]
    MalformedContent { source: Arc<dyn std::error::Error> },

    /// Error that occurs if the serialized structure contains invalid semantics.
    #[cfg(not(feature = "std"))]
    #[snafu(display("malformed content discovered"))]
    MalformedContent,

    /// Error that occurs if the serialized structure is incomplete.
    #[snafu(display("missing field: {}", field))]
    MissingField { field: String },

    /// Error in the bencode structure (e.g. a missing field and seperator).
    #[snafu(display("bencode encoding corrupted ({})", source))]
    StructureError {
        source: state_tracker::StructureError,
    },

    /// Error that occurs if the serialized structure contains an unexpected field.
    #[snafu(display("unexpected field: {}", field))]
    UnexpectedField { field: String },

    /// Error through an unexpected bencode token during deserialization.
    #[snafu(display("discovered {} but expected {}", expected, discovered))]
    UnexpectedToken {
        expected: String,
        discovered: String,
    },
}

pub trait ResultExt {
    fn context(self, context: impl Display) -> Self;
}

impl Error {
    pub fn context(mut self, context: impl Display) -> Self {
        if let Some(current) = self.context.as_mut() {
            *current = format!("{}.{}", context, current);
        } else {
            self.context = Some(context.to_string());
        }

        self
    }

    /// Raised when there is a general error while deserializing a type.
    /// The message should not be capitalized and should not end with a period.
    #[cfg(feature = "std")]
    pub fn malformed_content<SourceT>(source: SourceT) -> Self
    where
        SourceT: std::error::Error + Send + Sync + 'static,
    {
        let error = Arc::new(source);
        ErrorKind::MalformedContent { source: error }.into()
    }

    #[cfg(not(feature = "std"))]
    pub fn malformed_content<T>(_cause: T) -> Self {
        Self::from(ErrorKind::MalformedContent)
    }

    // Returns a `Error::MissingField` which contains the name of the field.
    pub fn missing_field(field_name: impl Display) -> Self {
        Error::from(ErrorKind::MissingField {
            field: field_name.to_string(),
        })
    }

    /// Returns a `Error::UnexpectedField` which contains the name of the field.
    pub fn unexpected_field(field_name: impl Display) -> Self {
        Error::from(ErrorKind::UnexpectedField {
            field: field_name.to_string(),
        })
    }

    /// Returns a `Error::UnexpectedElement` which contains a custom error message.
    pub fn unexpected_token(expected: impl Display, discovered: impl Display) -> Self {
        Error::from(ErrorKind::UnexpectedToken {
            expected: expected.to_string(),
            discovered: discovered.to_string(),
        })
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self {
            context: None,
            source: kind,
        }
    }
}

impl From<state_tracker::StructureError> for Error {
    fn from(error: state_tracker::StructureError) -> Self {
        Self::from(ErrorKind::StructureError { source: error })
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::malformed_content(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::malformed_content(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::malformed_content(err)
    }
}

impl<T> ResultExt for Result<T, Error> {
    fn context(self, context: impl Display) -> Self {
        self.map_err(|err| err.context(context))
    }
}
