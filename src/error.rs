use std::fmt::Display;

use crate::Loc;

/// `TokenizationError` represents errors that can occur during the tokenization process.
///
/// This enum provides detailed error types with associated location data (`Loc`), indicating
/// where in the input text the error occurred.
#[derive(Debug)]
pub enum TokenizationError {
    /// Indicates an invalid character parsing error.
    /// This variant is returned when a character sequence is not recognized as valid, based on
    /// the tokenizer configuration. For example, if the character sequence in question does not
    /// form a valid character, this error is raised.
    ///
    /// ### Fields
    /// - [Loc]: The line and column location in the input where the error occurred.
    NotAValidChar(Loc),
    /// Represents an unexpected digit separator error.
    /// This error is triggered when a digit separator appears in an invalid position within a
    /// number, or if a separator is encountered without surrounding digits.
    ///
    /// ### Fields
    /// - [Loc]: The line and column location in the input where the error occurred.
    UnexpectedDigitSeparator(Loc),
}

impl std::error::Error for TokenizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for TokenizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizationError::NotAValidChar(loc) => write!(f, "No valid character at {}", loc),
            TokenizationError::UnexpectedDigitSeparator(loc) => {
                write!(f, "Unexpected digit separator at {}", loc)
            }
        }
    }
}
