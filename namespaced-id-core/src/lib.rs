use thiserror::Error;

/// Validates that `string` is a valid namespaced id, returning [`Err`] if it is not.
///
/// # Errors
///
/// - [`ParseError::ExpectedNamespace`] if `string` has no namespace (is empty).
/// - [`ParseError::ExpectedId`] if `string` has no id (has no separator).
/// - [`ParseError::TooManySeparators`] if `string` has too many colons.
/// - [`ParseError::UnexpectedWhitespace`] if `string` has any whitespace.
pub const fn validate(string: &str) -> Result<(), ParseError> {
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

    match separators_found {
        0 if string.is_empty() => Err(ParseError::ExpectedNamespace),
        0 => Err(ParseError::ExpectedId),
        1 => Ok(()),
        count => Err(ParseError::TooManySeparators(count)),
    }
}

/// An error encountered while converting a string into a `NamespacedId`.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// No namespace was provided - the input was an empty string.
    #[error("expected a namespace, found an empty string")]
    ExpectedNamespace,
    /// No id was provided - the input had no separator.
    #[error("expected an id, found no separator (':')")]
    ExpectedId,
    /// There were too many separators (':' characters).
    #[error("expected only 1 separator (':'), found {0}")]
    TooManySeparators(usize),
    /// There was whitespace in the namespace or id.
    #[error("expected no whitespace, found some at index {0}")]
    UnexpectedWhitespace(usize),
}
