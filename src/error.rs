use std::fmt::Display;

use crate::Loc;

#[derive(Debug)]
pub enum TokenizationError {
    NotAValidChar(Loc),
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
