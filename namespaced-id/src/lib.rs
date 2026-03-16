use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

pub use namespaced_id_core::validate;

/// An error encountered while converting a string into a `NamespacedId`.
pub use namespaced_id_core::ParseError;

pub use namespaced_id_macros::ident;

/// A reference to a [`NamespacedId`], akin to a `str`.
///
/// This is identical to `str` in every way, except that it has the invariant of being a valid
/// [`NamespacedId`] (see [`Self::try_from_str`] for the requirements).
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NamespacedIdRef {
    inner: str,
}

impl NamespacedIdRef {
    /// Gets the string representation of this id.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::ident;
    ///
    /// let id = ident!("namespace:id");
    /// assert_eq!("namespace:id", id.as_str());
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &str {
        &self.inner
    }

    /// Losslessly converts `string` into a [`NamespacedIdRef`], or returns [`Err`] if it is not a
    /// valid id.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::NamespacedIdRef;
    ///
    /// let id = NamespacedIdRef::try_from_str("namespace:id")
    ///     .expect("id should be valid");
    /// assert_eq!("namespace:id", id.as_str());
    ///
    /// let id_res = NamespacedIdRef::try_from_str("invalid_id");
    /// assert!(id_res.is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ParseError::ExpectedNamespace`] if `string` has no namespace (is empty).
    /// - [`ParseError::ExpectedId`] if `string` has no id (has no separator).
    /// - [`ParseError::TooManySeparators`] if `string` has too many colons.
    /// - [`ParseError::UnexpectedWhitespace`] if `string` has any whitespace.
    pub const fn try_from_str(string: &str) -> Result<&Self, ParseError> {
        if let Err(err) = validate(string) {
            return Err(err);
        }

        Ok(Self::from_str_unchecked(string))
    }

    /// Converts `string` into a [`NamespacedIdRef`] without checking if it is a valid id.
    ///
    /// Note that this may cause panics down the line if the id is not valid, but it won't cause
    /// any undefined behavior.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use namespaced_id::NamespacedIdRef;
    ///
    /// let id = NamespacedIdRef::from_str_unchecked("invalid_id");
    /// // panics here, as `id` is invalid (and so it doesn't have an id)
    /// let _ = id.id();
    /// ```
    #[must_use]
    pub const fn from_str_unchecked(string: &str) -> &Self {
        // SAFETY: NamespacedIdRef is #[repr(transparent)]
        // NOTE: safety is not upheld by caller - this is always safe
        unsafe { &*(std::ptr::from_ref(string) as *const Self) }
    }

    /// Returns the portion of the id before the colon.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::ident;
    ///
    /// assert_eq!("namespace", ident!("namespace:id").namespace());
    /// ```
    #[must_use]
    pub const fn namespace(&self) -> &str {
        let namespace_len = self.namespace_len();
        let (namespace, _) = self.inner.split_at(namespace_len);
        namespace
    }

    /// Returns the portion of the id after the colon.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::ident;
    ///
    /// assert_eq!("id", ident!("namespace:id").id());
    /// ```
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
        write!(f, "ident!(\"{}\")", &self.as_str())
    }
}

/// An owned id that is prefixed with a namespace, like `<namespace>:<id>`.
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`NamespacedId`] (see [`NamespacedIdRef::try_from_str`] for the requirements).
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NamespacedId {
    inner: Box<NamespacedIdRef>,
}

impl NamespacedId {
    /// Creates a new [`NamespacedId`] from a separated `namespace` and `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::NamespacedId;
    ///
    /// let id = NamespacedId::try_new("namespace", "id")
    ///     .expect("id should be valid");
    /// assert_eq!("namespace:id", id.as_str());
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ParseError::TooManySeparators`] if either component has a colon.
    /// - [`ParseError::UnexpectedWhitespace`] if either component has whitespace.
    pub fn try_new(namespace: &str, id: &str) -> Result<Self, ParseError> {
        let string = format!("{namespace}:{id}");
        let boxed = Box::<str>::from(string);
        Self::try_from_box(boxed)
    }

    /// Creates a new [`NamespacedId`] from a [`Box<str>`].
    ///
    /// # Errors
    ///
    /// - See [`NamespacedIdRef::try_from_str`].
    pub fn try_from_box(string: Box<str>) -> Result<Self, ParseError> {
        validate(&string)?;
        Ok(Self::from_box_unchecked(string))
    }

    /// Creates a new [`NamespacedId`] by allocating a [`&str`].
    ///
    /// # Errors
    ///
    /// - See [`NamespacedIdRef::try_from_str`].
    pub fn try_from_str(string: &str) -> Result<Self, ParseError> {
        validate(string)?;
        Ok(Self::from_box_unchecked(Box::<str>::from(string)))
    }

    /// Converts `string` into a [`NamespacedIdRef`] without checking if it is a valid id.
    ///
    /// Note that this may cause panics down the line if the id is not valid, but it won't cause
    /// any undefined behavior.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use namespaced_id::NamespacedId;
    ///
    /// let id = NamespacedId::from_box_unchecked(From::from("invalid_id"));
    /// // panics here, as `id` is invalid (and so it doesn't have a namespace)
    /// let _ = id.namespace();
    /// ```
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
        write!(f, "*ident!(\"{}\")", &self.as_str())
    }
}

// deref-style impls

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

impl ToOwned for NamespacedIdRef {
    type Owned = NamespacedId;
    fn to_owned(&self) -> Self::Owned {
        NamespacedId::from_box_unchecked(Box::<str>::from(self.as_str()))
    }
}

// Borrow<str> impls

impl Borrow<str> for NamespacedIdRef {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for NamespacedIdRef {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for NamespacedId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for NamespacedId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// PartialEq and PartialOrd impls

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

impl TryFrom<&str> for NamespacedId {
    type Error = ParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl TryFrom<Box<str>> for NamespacedId {
    type Error = ParseError;
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::try_from_box(value)
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
    use crate as namespaced_id;
    use crate::{NamespacedId, NamespacedIdRef, ParseError, ident};

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
