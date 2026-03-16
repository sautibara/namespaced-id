use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

use thiserror::Error;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NamespacedIdRef {
    inner: str,
}

#[macro_export]
macro_rules! ident {
    ($str:literal $(,)?) => {
        const {
            match $crate::NamespacedIdRef::try_from_str($str) {
                Ok(val) => val,
                Err(_) => panic!(),
            }
        }
    };
}

impl NamespacedIdRef {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        &self.inner
    }

    pub const fn try_from_str(string: &str) -> Result<&Self, ParseError> {
        if let Err(err) = validate(string) {
            return Err(err);
        }

        Ok(Self::from_str_unchecked(string))
    }

    #[must_use]
    pub const fn from_str_unchecked(string: &str) -> &Self {
        // SAFETY: NamespacedIdRef is #[repr(transparent)]
        // NOTE: safety is not upheld by caller - this is always safe
        unsafe { &*(std::ptr::from_ref(string) as *const Self) }
    }

    #[must_use]
    pub const fn namespace(&self) -> &str {
        let namespace_len = self.namespace_len();
        let (namespace, _) = self.inner.split_at(namespace_len);
        namespace
    }

    #[must_use]
    pub const fn id(&self) -> &str {
        let namespace_len = self.namespace_len();
        let (_, id) = self.inner.split_at(namespace_len + 1);
        id
    }

    const fn namespace_len(&self) -> usize {
        let mut i = 0;
        while i < self.inner.len() {
            let byte = self.inner.as_bytes()[i];
            if byte == b':' {
                return i;
            }

            i += 1;
        }

        unreachable!()
    }
}

impl Display for NamespacedIdRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl Debug for NamespacedIdRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamespacedIdRef(\"{}\")", &self.as_str())
    }
}

impl ToOwned for NamespacedIdRef {
    type Owned = NamespacedId;
    fn to_owned(&self) -> Self::Owned {
        NamespacedId::from_box_unchecked(Box::<str>::from(self.as_str()))
    }
}

impl Borrow<str> for NamespacedIdRef {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NamespacedId {
    inner: Box<NamespacedIdRef>,
}

impl NamespacedId {
    pub fn try_new(namespace: &str, id: &str) -> Result<Self, ParseError> {
        let string = format!("{namespace}:{id}");
        let boxed = Box::<str>::from(string);
        Self::try_from_box(boxed)
    }

    pub fn try_from_box(string: Box<str>) -> Result<Self, ParseError> {
        validate(&string)?;
        Ok(Self::from_box_unchecked(string))
    }

    pub fn try_from_str(string: &str) -> Result<Self, ParseError> {
        validate(string)?;
        Ok(Self::from_box_unchecked(Box::<str>::from(string)))
    }

    #[must_use]
    pub const fn from_box_unchecked(string: Box<str>) -> Self {
        // SAFETY: NamespacedId is #[repr(transparent)]
        // NOTE: safety is not upheld by caller - this is always safe
        unsafe { std::mem::transmute(string) }
    }
}

impl Display for NamespacedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl Debug for NamespacedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamespacedId(\"{}\")", &self.as_str())
    }
}

impl Deref for NamespacedId {
    type Target = NamespacedIdRef;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Borrow<NamespacedIdRef> for NamespacedId {
    fn borrow(&self) -> &NamespacedIdRef {
        &self.inner
    }
}

impl Clone for NamespacedId {
    fn clone(&self) -> Self {
        self.inner.to_owned()
    }
}

impl From<&NamespacedIdRef> for NamespacedId {
    fn from(value: &NamespacedIdRef) -> Self {
        value.to_owned()
    }
}

impl Borrow<str> for NamespacedId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[allow(unstable_name_collisions)]
mod cmp_impls {
    use super::{NamespacedId, NamespacedIdRef};

    trait StrAsStr {
        fn as_str(&self) -> &str;
    }

    impl StrAsStr for str {
        fn as_str(&self) -> &str {
            self
        }
    }

    // accept comparing any combination of NamespacedId, NamespacedIdRef, and str

    macro_rules! cmp_impls {
        ($left:ty, $right:ty) => {
            impl PartialEq<$right> for $left {
                fn eq(&self, right: &$right) -> bool {
                    self.as_str() == right.as_str()
                }
            }

            impl PartialOrd<$right> for $left {
                fn partial_cmp(&self, other: &$right) -> Option<std::cmp::Ordering> {
                    self.as_str().partial_cmp(other.as_str())
                }
            }
        };
    }

    cmp_impls!(NamespacedId, &NamespacedId);
    cmp_impls!(NamespacedId, NamespacedIdRef);
    cmp_impls!(NamespacedId, &NamespacedIdRef);
    cmp_impls!(NamespacedId, str);
    cmp_impls!(NamespacedId, &str);

    cmp_impls!(&NamespacedId, NamespacedId);
    cmp_impls!(&NamespacedId, NamespacedIdRef);
    cmp_impls!(&NamespacedId, str);

    cmp_impls!(NamespacedIdRef, NamespacedId);
    cmp_impls!(NamespacedIdRef, &NamespacedId);
    cmp_impls!(NamespacedIdRef, &NamespacedIdRef);
    cmp_impls!(NamespacedIdRef, str);
    cmp_impls!(NamespacedIdRef, &str);

    cmp_impls!(&NamespacedIdRef, NamespacedId);
    cmp_impls!(&NamespacedIdRef, NamespacedIdRef);
    cmp_impls!(&NamespacedIdRef, str);

    cmp_impls!(str, NamespacedId);
    cmp_impls!(str, &NamespacedId);
    cmp_impls!(str, NamespacedIdRef);
    cmp_impls!(str, &NamespacedIdRef);

    cmp_impls!(&str, NamespacedId);
    cmp_impls!(&str, NamespacedIdRef);
}

/// An error encountered while parsing a [`NamespacedId`]
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("expected a namespace, found an empty string")]
    ExpectedNamespace,
    #[error("expected an id, found no separator (':')")]
    ExpectedId,
    #[error("expected only 1 separator (':'), found {0}")]
    TooManySeparators(usize),
    #[error("expected no whitespace, found some at index {0}")]
    UnexpectedWhitespace(usize),
}

impl<'a> TryFrom<&'a str> for &'a NamespacedIdRef {
    type Error = ParseError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        NamespacedIdRef::try_from_str(value)
    }
}

impl FromStr for NamespacedId {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s)
    }
}

const fn validate(string: &str) -> Result<(), ParseError> {
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

#[cfg(feature = "serde")]
impl Serialize for NamespacedId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for NamespacedId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(NamespacedIdVisitor)
    }
}

#[cfg(feature = "serde")]
struct NamespacedIdVisitor;
#[cfg(feature = "serde")]
impl Visitor<'_> for NamespacedIdVisitor {
    type Value = NamespacedId;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "a namespaced id string in the form of '<namespace>:<id>'"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::{NamespacedId, NamespacedIdRef, ParseError};

    #[test]
    fn roundtrip_check() {
        assert_eq!(
            Ok("namespace:id"),
            NamespacedIdRef::try_from_str("namespace:id").map(NamespacedIdRef::as_str)
        );
    }

    #[test]
    fn expected_namespace_error() {
        assert_eq!(
            Err(ParseError::ExpectedNamespace),
            NamespacedIdRef::try_from_str("")
        );
    }

    #[test]
    fn expected_id_error() {
        assert_eq!(
            Err(ParseError::ExpectedId),
            NamespacedIdRef::try_from_str("a")
        );
    }

    #[test]
    fn too_many_separators_error() {
        assert_eq!(
            Err(ParseError::TooManySeparators(2)),
            NamespacedIdRef::try_from_str("a:b:c")
        );
    }

    #[test]
    fn unexpected_whitespace_error() {
        assert_eq!(
            Err(ParseError::UnexpectedWhitespace(4)),
            NamespacedIdRef::try_from_str("name space:id")
        );
    }

    #[test]
    fn extract_namespace() {
        assert_eq!("namespace", ident!("namespace:id").namespace());
    }

    #[test]
    fn extract_id() {
        assert_eq!("id", ident!("namespace:id").id());
    }

    #[test]
    #[should_panic = "unreachable"]
    fn namespace_panics_without_separator() {
        let _ = NamespacedIdRef::from_str_unchecked("no_separator").namespace();
    }

    #[test]
    #[should_panic = "unreachable"]
    fn id_panics_without_separator() {
        let _ = NamespacedIdRef::from_str_unchecked("no_separator").id();
    }

    #[test]
    fn empty_namespace_and_id() {
        match NamespacedIdRef::try_from_str(":") {
            Ok(id) => {
                assert_eq!("", id.namespace());
                assert_eq!("", id.id());
            }
            Err(err) => {
                panic!("{err}");
            }
        }
    }

    #[test]
    fn owned_roundtrip() {
        assert_eq!("namespace:id", ident!("namespace:id").to_owned().as_str());
    }

    #[test]
    fn try_new() {
        match NamespacedId::try_new("namespace", "id") {
            Ok(id) => {
                assert_eq!("namespace:id", id);
            }
            Err(err) => {
                panic!("{err}");
            }
        }
    }
}
