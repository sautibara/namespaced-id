use thiserror::Error;

/// Validates that `string` is a valid namespaced id, returning [`Err`] if it is not.
///
/// # Errors
///
/// - [`ParseError::UnexpectedComponentCount`] if `string` has the wrong number of components.
/// - [`ParseError::UnexpectedWhitespace`] if `string` has any whitespace.
pub const fn validate<const N: usize>(string: &str) -> Result<(), ParseError<N>> {
    let mut i = 0;
    let mut separators_found = 0;
    while i < string.len() {
        let byte = string.as_bytes()[i];
        let character = if byte.is_ascii() {
            byte as char
        } else {
            continue;
        };

        if character == ':' {
            separators_found += 1;
        } else if character.is_whitespace() {
            return Err(ParseError::UnexpectedWhitespace(i));
        }

        i += 1;
    }

    let component_count = separators_found + 1;
    if component_count == N {
        Ok(())
    } else if string.is_empty() {
        Err(ParseError::UnexpectedComponentCount(0))
    } else {
        Err(ParseError::UnexpectedComponentCount(component_count))
    }
}

/// An error encountered while converting a string into a `NamespacedId`.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError<const N: usize> {
    /// There was an invalid amount of components.
    #[error("expected {} components ({} colon(s)), found {}", N, N - 1, .0)]
    UnexpectedComponentCount(usize),
    /// There was whitespace in the namespace or id.
    #[error("expected no whitespace, found some at index {0}")]
    UnexpectedWhitespace(usize),
}
