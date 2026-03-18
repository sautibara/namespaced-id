use thiserror::Error;

const fn is_valid_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || byte == b'.'
        || byte == b'_'
        || byte == b'-'
        || byte == b'/'
}

/// Validates that `string` is a valid delimited id with `N` components, returning [`Err`] if not.
///
/// # Errors
///
/// - [`ParseError::UnexpectedComponentCount`] if `string` has the wrong number of components.
/// - [`ParseError::UnexpectedCharacter`] if `string` has any characters not in `[a-z0-9-_./]`.
pub const fn validate<const N: usize>(string: &str) -> Result<(), ParseError<N>> {
    let mut i = 0;
    let mut separators_found = 0;
    while i < string.len() {
        let byte = string.as_bytes()[i];
        if byte == b':' {
            separators_found += 1;
        } else if !is_valid_byte(byte) {
            return Err(ParseError::UnexpectedCharacter(i));
        }

        i += 1;
    }

    let component_count = separators_found + 1;
    if component_count == N || N == 0 && string.is_empty() {
        Ok(())
    } else if string.is_empty() {
        Err(ParseError::UnexpectedComponentCount(0))
    } else {
        Err(ParseError::UnexpectedComponentCount(component_count))
    }
}

/// An error encountered while converting a string into a delimited id with `N` components.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError<const N: usize> {
    /// There was an invalid amount of components.
    #[error("expected {} components ({} colon(s)), found {}", N, N - 1, .0)]
    UnexpectedComponentCount(usize),
    /// There was an unexpected character in the namespace or id - one not in `[a-z0-9-_./]`.
    #[error("expected only characters in [a-z0-9-_./], found an unexpected one at index {0}")]
    UnexpectedCharacter(usize),
}
