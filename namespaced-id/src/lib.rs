use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
};

pub use namespaced_id_core::validate;

/// An error encountered while converting a string into a `NamespacedId`.
pub use namespaced_id_core::ParseError;

/// Returns a [`&'static NamespacedIdRef`](NamespacedIdRef) of the given namespaced id string
/// literal.
///
/// # Examples
///
/// ```
/// use namespaced_id::ident;
/// let id = ident!("namespace:id");
/// assert_eq!("namespace:id", id.as_str());
/// ```
///
/// Compilation fails if the identifier is not a valid [`NamespacedId`].
///
/// ```compile_fail
/// namespaced_id::ident!("invalid_id");
/// ```
pub use namespaced_id_macros::ident;

/// Returns a [`&'static OperationIdRef`](OperationIdRef) of the given operation id string
/// literal.
///
/// # Examples
///
/// ```
/// use namespaced_id::op_ident;
/// let id = op_ident!("namespace:id:operation");
/// assert_eq!("namespace:id:operation", id.as_str());
/// ```
///
/// Compilation fails if the identifier is not a valid [`OperationId`].
///
/// ```compile_fail
/// namespaced_id::op_ident!("invalid_id");
/// ```
///
/// ```compile_fail
/// namespaced_id::op_ident!("invalid:id");
/// ```
pub use namespaced_id_macros::op_ident;

/// A reference to a [`DelimitedId`], akin to a `str`.
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`NamespacedId`] (see [`validate`] for the requirements).
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DelimitedIdRef<const N: usize> {
    inner: str,
}

impl<const N: usize> DelimitedIdRef<N> {
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
    /// - [`ParseError::UnexpectedComponentCount`] if `string` doesn't have two components
    ///   (one ':').
    /// - [`ParseError::UnexpectedWhitespace`] if `string` has any whitespace.
    pub const fn try_from_str(string: &str) -> Result<&Self, ParseError<N>> {
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
    /// // Panics here, as `id` is invalid (and so it doesn't have an id).
    /// let _ = id.id();
    /// ```
    #[must_use]
    pub const fn from_str_unchecked(string: &str) -> &Self {
        // SAFETY: NamespacedIdRef is #[repr(transparent)]
        // NOTE: safety is not upheld by caller - this is always safe
        unsafe { &*(std::ptr::from_ref(string) as *const Self) }
    }

    const fn delimiter_indicies(&self) -> [usize; N] {
        let mut indicies = [0; _];

        let mut i = 0;
        let mut array_index = 0;
        while i < self.inner.len() {
            let byte = self.inner.as_bytes()[i];
            if byte == b':' {
                indicies[array_index] = i;
                array_index += 1;
            }

            i += 1;
        }

        debug_assert!(array_index + 1 == N, "incorrect separator count");

        indicies[array_index] = i;
        indicies
    }
}

impl<'a, const N: usize> TryFrom<&'a str> for &'a DelimitedIdRef<N> {
    type Error = ParseError<N>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        DelimitedIdRef::try_from_str(value)
    }
}

/// An id that is made up of `N` different components that are all separated by `:` delimiters.
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`NamespacedId`] (see [`validate`] for the requirements).
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DelimitedId<const N: usize> {
    inner: Box<DelimitedIdRef<N>>,
}

impl<const N: usize> DelimitedId<N> {
    /// Creates a new [`NamespacedId`] from a [`Box<str>`].
    ///
    /// # Errors
    ///
    /// - See [`validate`].
    pub fn try_from_box(string: Box<str>) -> Result<Self, ParseError<N>> {
        validate(&string)?;
        Ok(Self::from_box_unchecked(string))
    }

    /// Creates a new [`NamespacedId`] by allocating a [`&str`].
    ///
    /// # Errors
    ///
    /// - See [`validate`].
    pub fn try_from_str(string: &str) -> Result<Self, ParseError<N>> {
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
    /// // Panics here, as `id` is invalid (and so it doesn't have a namespace).
    /// let _ = id.namespace();
    /// ```
    #[must_use]
    pub const fn from_box_unchecked(string: Box<str>) -> Self {
        // SAFETY: NamespacedId is #[repr(transparent)]
        // NOTE: safety is not upheld by caller - this is always safe
        unsafe { std::mem::transmute(string) }
    }
}

impl<const N: usize> TryFrom<&str> for DelimitedId<N> {
    type Error = ParseError<N>;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl<const N: usize> TryFrom<Box<str>> for DelimitedId<N> {
    type Error = ParseError<N>;
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::try_from_box(value)
    }
}

impl<const N: usize> FromStr for DelimitedId<N> {
    type Err = ParseError<N>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(s)
    }
}

// display impls

impl<const N: usize> Display for DelimitedIdRef<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

impl<const N: usize> Display for DelimitedId<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

// deref-style impls

impl<const N: usize> Deref for DelimitedId<N> {
    type Target = DelimitedIdRef<N>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const N: usize> Borrow<DelimitedIdRef<N>> for DelimitedId<N> {
    fn borrow(&self) -> &DelimitedIdRef<N> {
        &self.inner
    }
}

impl<const N: usize> Clone for DelimitedId<N> {
    fn clone(&self) -> Self {
        self.inner.to_owned()
    }
}

impl<const N: usize> From<&DelimitedIdRef<N>> for DelimitedId<N> {
    fn from(value: &DelimitedIdRef<N>) -> Self {
        value.to_owned()
    }
}

impl<const N: usize> ToOwned for DelimitedIdRef<N> {
    type Owned = DelimitedId<N>;
    fn to_owned(&self) -> Self::Owned {
        DelimitedId::from_box_unchecked(Box::<str>::from(self.as_str()))
    }
}

// Borrow<str> impls

impl<const N: usize> Borrow<str> for DelimitedIdRef<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for DelimitedIdRef<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> Borrow<str> for DelimitedId<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for DelimitedId<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// PartialEq and PartialOrd impls

#[allow(unstable_name_collisions)]
mod cmp_impls {
    use super::{DelimitedId, DelimitedIdRef};

    trait StrAsStr {
        fn as_str(&self) -> &str;
    }

    impl StrAsStr for str {
        fn as_str(&self) -> &str {
            self
        }
    }

    // accept comparing any combination of DelimitedId, DelimitedIdRef, and str

    macro_rules! cmp_impls {
        ($left:ty, $right:ty) => {
            impl<const N: usize> PartialEq<$right> for $left {
                fn eq(&self, right: &$right) -> bool {
                    self.as_str() == right.as_str()
                }
            }

            impl<const N: usize> PartialOrd<$right> for $left {
                fn partial_cmp(&self, other: &$right) -> Option<std::cmp::Ordering> {
                    self.as_str().partial_cmp(other.as_str())
                }
            }
        };
    }

    cmp_impls!(DelimitedId<N>, &DelimitedId<N>);
    cmp_impls!(DelimitedId<N>, DelimitedIdRef<N>);
    cmp_impls!(DelimitedId<N>, &DelimitedIdRef<N>);
    cmp_impls!(DelimitedId<N>, str);
    cmp_impls!(DelimitedId<N>, &str);

    cmp_impls!(&DelimitedId<N>, DelimitedId<N>);
    cmp_impls!(&DelimitedId<N>, DelimitedIdRef<N>);
    cmp_impls!(&DelimitedId<N>, str);

    cmp_impls!(DelimitedIdRef<N>, DelimitedId<N>);
    cmp_impls!(DelimitedIdRef<N>, &DelimitedId<N>);
    cmp_impls!(DelimitedIdRef<N>, &DelimitedIdRef<N>);
    cmp_impls!(DelimitedIdRef<N>, str);
    cmp_impls!(DelimitedIdRef<N>, &str);

    cmp_impls!(&DelimitedIdRef<N>, DelimitedId<N>);
    cmp_impls!(&DelimitedIdRef<N>, DelimitedIdRef<N>);
    cmp_impls!(&DelimitedIdRef<N>, str);

    cmp_impls!(str, DelimitedId<N>);
    cmp_impls!(str, &DelimitedId<N>);
    cmp_impls!(str, DelimitedIdRef<N>);
    cmp_impls!(str, &DelimitedIdRef<N>);

    cmp_impls!(&str, DelimitedId<N>);
    cmp_impls!(&str, DelimitedIdRef<N>);
}

#[cfg(feature = "serde")]
impl<const N: usize> Serialize for DelimitedId<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> Deserialize<'de> for DelimitedId<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(DelimitedIdVisitor::<N>)
    }
}

#[cfg(feature = "serde")]
struct DelimitedIdVisitor<const N: usize>;
#[cfg(feature = "serde")]
impl<const N: usize> Visitor<'_> for DelimitedIdVisitor<N> {
    type Value = DelimitedId<N>;

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

/// A reference to a [`NamespacedId`], akin to a `str`.
///
/// This is identical to `str` in every way, except that it has the invariant of being a valid
/// [`NamespacedId`] (see [`validate`] for the requirements).
pub type NamespacedIdRef = DelimitedIdRef<2>;

impl NamespacedIdRef {
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
        let [namespace_len, _] = self.delimiter_indicies();
        namespace_len
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
/// [`NamespacedId`] (see [`validate`] for the requirements).
pub type NamespacedId = DelimitedId<2>;

impl NamespacedId {
    /// Creates a new [`NamespacedId`] from a separated `namespace` and `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::NamespacedId;
    ///
    /// let id = NamespacedId::new("namespace", "id")
    ///     .expect("id should be valid");
    /// assert_eq!("namespace:id", id.as_str());
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ParseError::UnexpectedComponentCount`] if either component has a colon.
    /// - [`ParseError::UnexpectedWhitespace`] if either component has whitespace.
    pub fn new(namespace: &str, id: &str) -> Result<Self, ParseError<2>> {
        let string = format!("{namespace}:{id}");
        let boxed = Box::<str>::from(string);
        Self::try_from_box(boxed)
    }
}

impl Debug for NamespacedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*ident!(\"{}\")", &self.as_str())
    }
}

/// A reference to a [`OperationId`], akin to a `str`.
///
/// This is identical to `str` in every way, except that it has the invariant of being a valid
/// [`NamespacedId`] (see [`validate`] for the requirements).
pub type OperationIdRef = DelimitedIdRef<3>;

impl OperationIdRef {
    /// Returns the first two components of the id - the namespaced id.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::{op_ident, ident};
    ///
    /// assert_eq!(ident!("namespace:id"), op_ident!("namespace:id:operation").namespaced_id());
    /// ```
    #[must_use]
    pub const fn namespaced_id(&self) -> &NamespacedIdRef {
        let [_, second_colon, _] = self.delimiter_indicies();
        let (namespaced_id, _) = self.inner.split_at(second_colon);
        NamespacedIdRef::from_str_unchecked(namespaced_id)
    }

    /// Returns the first component of the id - the namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::op_ident;
    ///
    /// assert_eq!("namespace", op_ident!("namespace:id:operation").namespace());
    /// ```
    #[must_use]
    pub const fn namespace(&self) -> &str {
        self.namespaced_id().namespace()
    }

    /// Returns the second component of the id - the id.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::op_ident;
    ///
    /// assert_eq!("id", op_ident!("namespace:id:operation").id());
    /// ```
    #[must_use]
    pub const fn id(&self) -> &str {
        self.namespaced_id().id()
    }

    /// Returns the third component of the id - the operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::op_ident;
    ///
    /// assert_eq!("operation", op_ident!("namespace:id:operation").operation());
    /// ```
    #[must_use]
    pub const fn operation(&self) -> &str {
        let [_, second_colon, _] = self.delimiter_indicies();
        let (_, operation) = self.inner.split_at(second_colon + 1);
        operation
    }
}

impl Debug for OperationIdRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "op_ident!(\"{}\")", &self.as_str())
    }
}

/// A [`NamespacedId`] plus an operation, like `<namespace>:<id>:<operation>`.
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`OperationId`] (see [`validate`] for the requirements).
pub type OperationId = DelimitedId<3>;

impl OperationId {
    /// Creates a new [`OperationId`] from a separated `namespace`, `id`, and `operation`.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::OperationId;
    ///
    /// let id = OperationId::new("namespace", "id", "operation")
    ///     .expect("id should be valid");
    /// assert_eq!("namespace:id:operation", id.as_str());
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ParseError::UnexpectedComponentCount`] if either component has a colon.
    /// - [`ParseError::UnexpectedWhitespace`] if either component has whitespace.
    pub fn new(namespace: &str, id: &str, operation: &str) -> Result<Self, ParseError<3>> {
        let string = format!("{namespace}:{id}:{operation}");
        let boxed = Box::<str>::from(string);
        Self::try_from_box(boxed)
    }
}

impl Debug for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*op_ident!(\"{}\")", &self.as_str())
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
            Err(ParseError::UnexpectedComponentCount(0)),
            NamespacedIdRef::try_from_str("")
        );
    }

    #[test]
    fn expected_id_error() {
        assert_eq!(
            Err(ParseError::UnexpectedComponentCount(1)),
            NamespacedIdRef::try_from_str("a")
        );
    }

    #[test]
    fn too_many_separators_error() {
        assert_eq!(
            Err(ParseError::UnexpectedComponentCount(3)),
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
    #[should_panic = "incorrect separator count"]
    fn namespace_panics_without_separator() {
        let _ = NamespacedIdRef::from_str_unchecked("no_separator").namespace();
    }

    #[test]
    #[should_panic = "incorrect separator count"]
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
        match NamespacedId::new("namespace", "id") {
            Ok(id) => {
                assert_eq!("namespace:id", id);
            }
            Err(err) => {
                panic!("{err}");
            }
        }
    }
}
