//! A crate that defines types that identify data in a human-readable way.
//!
//! The most common example is the [`NamespacedId`], which consists of a namespace and path
//! separated by a colon (ex: `namespace:path`). This leads to an id that is somewhat resistant to
//! collisions, but is still human-readable.
//!
//! All ids are based on a [`DelimitedId`], which is a generic id with a constant number of
//! components separated by colons. It is a wrapper over a [`Box<str>`], ensuring that its interior
//! is a valid id. Every id also has a corresponding wrapper over a [`str`], backed by a
//! [`DelimitedIdRef`]. As both are wrappers over `Box<str>` and `str` respectively, they can be
//! losslessly converted between the two without having to allocate.
//!
//! There are three main id types:
//! - [`IdComponent`] (ex: `component`)
//!     - Used to ensure that the backing string is valid as a single component of a larger id
//!       (no separators or invalid characters).
//! - [`NamespacedId`] (ex: `namespace:path`)
//!     - Used to identify different objects.
//! - [`OperationId`] (ex: `namespace:path:operation`)
//!     - Used to identify operations that refer to a specific identified object.
//!
//! Each main id type also has a corresponding macro to generate and validate a static reference at
//! compile time.
//! - `IdComponent` has [`ident_component!`]
//! - `NamespacedId` has [`ident!`]
//! - `OperationId` has [`op_ident!`]
//!
//! # Examples
//!
//! ```
//! use namespaced_id::ident;
//!
//! let id = ident!("namespace:path");
//! assert_eq!("namespace", id.namespace());
//! assert_eq!("path", id.path());
//! ```
//!
//! ```
//! use namespaced_id::NamespacedIdRef;
//!
//! let runtime_str = "other:id";
//! let id = NamespacedIdRef::new(runtime_str)
//!     .expect("runtime_str is a valid id");
//!
//! assert_eq!("other", id.namespace());
//! assert_eq!("id", id.path());
//! ```
//!
//! # Origin
//!
//! The original idea for this library came from Minecraft's namespaced id, which is very very
//! similar. The game uses it for everything, and I came to like it a lot as I used it, so I made
//! this library to use it in my projects. Notice that it isn't a fully faithful reproduction,
//! though, as this library allows `/` characters in the namespace.

use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    ops::{Add, Deref},
    str::FromStr,
    sync::Arc,
};

use self as namespaced_id;

const fn substring_const(string: &str, start: usize, end: usize) -> &str {
    let (string, _) = string.split_at(end);
    let (_, string) = string.split_at(start);
    string
}

pub use namespaced_id_core::validate;

pub use namespaced_id_core::ParseError;

/// Returns a [`&'static NamespacedIdRef`](NamespacedIdRef) of the given namespaced id string
/// literal.
///
/// # Examples
///
/// ```
/// use namespaced_id::ident;
/// let id = ident!("namespace:path");
/// assert_eq!("namespace:path", id.as_str());
/// ```
///
/// Compilation fails if the identifier is not a valid [`NamespacedId`].
///
/// ```compile_fail
/// namespaced_id::ident!("invalid_id");
/// ```
#[cfg(doc)]
#[macro_export]
macro_rules! ident {
    ($str:literal) => {
        todo!()
    };
}
#[cfg(not(doc))]
pub use namespaced_id_macros::ident;

/// Returns a [`&'static OperationIdRef`](OperationIdRef) of the given operation id string
/// literal.
///
/// # Examples
///
/// ```
/// use namespaced_id::op_ident;
/// let id = op_ident!("namespace:path:operation");
/// assert_eq!("namespace:path:operation", id.as_str());
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
#[cfg(doc)]
#[macro_export]
macro_rules! op_ident {
    ($str:literal) => {
        todo!()
    };
}
#[cfg(not(doc))]
pub use namespaced_id_macros::op_ident;

/// Returns a [`&'static IdComponentRef`](IdComponentRef) of the given id component string literal.
///
/// # Examples
///
/// ```
/// use namespaced_id::ident_component;
/// let id = ident_component!("namespace");
/// assert_eq!("namespace", id.as_str());
/// ```
///
/// Compilation fails if the identifier is not a valid [`IdComponent`].
///
/// ```compile_fail
/// namespaced_id::op_ident!("invalid:id");
/// ```
///
/// ```compile_fail
/// namespaced_id::op_ident!("invalid id");
/// ```
#[cfg(doc)]
#[macro_export]
macro_rules! ident_component {
    ($str:literal) => {
        todo!()
    };
}
#[cfg(not(doc))]
pub use namespaced_id_macros::ident_component;

/// A reference to a [`DelimitedId`], akin to a `str`.
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`DelimitedId`] (see [`validate`] for the requirements).
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
    /// let id = ident!("namespace:path");
    /// assert_eq!("namespace:path", id.as_str());
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &str {
        &self.inner
    }

    /// Returns an array of all components of this id, in order, without any delimiters.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::op_ident;
    ///
    /// let id = op_ident!("namespace:path:operation");
    /// assert_eq!(["namespace", "path", "operation"], id.components());
    /// ```
    #[must_use]
    pub const fn components(&self) -> [&IdComponentRef; N] {
        let mut components = [ident_component!(""); _];
        if N == 0 {
            return components;
        }

        let end_indicies = self.component_end_indicies();
        components[0] =
            IdComponentRef::from_str_unchecked(substring_const(self.as_str(), 0, end_indicies[0]));

        let mut i = 1;
        while i < N {
            components[i] = IdComponentRef::from_str_unchecked(substring_const(
                self.as_str(),
                end_indicies[i - 1] + 1,
                end_indicies[i],
            ));
            i += 1;
        }

        components
    }

    /// Losslessly converts `string` into a [`DelimitedIdRef`], or returns [`Err`] if it is not a
    /// valid id.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::NamespacedIdRef;
    ///
    /// let id = NamespacedIdRef::new("namespace:path")
    ///     .expect("id should be valid");
    /// assert_eq!("namespace:path", id.as_str());
    ///
    /// let id_res = NamespacedIdRef::new("invalid_id");
    /// assert!(id_res.is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// - [`ParseError::UnexpectedComponentCount`] if `string` doesn't have the correct number
    ///   of components.
    /// - [`ParseError::UnexpectedCharacter`] if `string` has a character not in `[a-z0-9-_./]`.
    pub const fn new(string: &str) -> Result<&Self, ParseError<N>> {
        if let Err(err) = validate(string) {
            return Err(err);
        }

        Ok(Self::from_str_unchecked(string))
    }

    /// Converts `string` into a [`DelimitedIdRef`] without checking if it is a valid id.
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
    /// let _ = id.path();
    /// ```
    #[must_use]
    pub const fn from_str_unchecked(string: &str) -> &Self {
        // SAFETY: DelimitedIdRef is #[repr(transparent)]
        // NOTE: safety is not upheld by caller - this is always safe
        unsafe { &*(std::ptr::from_ref(string) as *const Self) }
    }

    const fn component_end_indicies(&self) -> [usize; N] {
        let mut indicies = [0; _];
        if N == 0 {
            return indicies;
        }

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
        DelimitedIdRef::new(value)
    }
}

/// An id that is made up of `N` different components that are all separated by `:` delimiters.
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`DelimitedId`] (see [`validate`] for the requirements).
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DelimitedId<const N: usize> {
    inner: Box<DelimitedIdRef<N>>,
}

impl<const N: usize> DelimitedId<N> {
    /// Creates a new [`DelimitedId`] from a [`String`].
    ///
    /// # Errors
    ///
    /// - See [`validate`].
    pub fn try_from_string(string: String) -> Result<Self, ParseError<N>> {
        validate(&string)?;
        Ok(Self::from_box_unchecked(string.into_boxed_str()))
    }

    /// Creates a new [`DelimitedId`] from a [`Box<str>`].
    ///
    /// # Errors
    ///
    /// - See [`validate`].
    pub fn try_from_box(string: Box<str>) -> Result<Self, ParseError<N>> {
        validate(&string)?;
        Ok(Self::from_box_unchecked(string))
    }

    /// Creates a new [`DelimitedId`] by allocating a [`&str`].
    ///
    /// # Errors
    ///
    /// - See [`validate`].
    pub fn try_from_str(string: &str) -> Result<Self, ParseError<N>> {
        validate(string)?;
        Ok(Self::from_box_unchecked(Box::<str>::from(string)))
    }

    /// Converts `string` into a [`DelimitedId`] without checking if it is a valid id.
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

/// An id that is made up of `N` different components that are all separated by `:` delimiters.
///
/// This is identical to `Arc<str>` in every way, except that it has the invariant of being a valid
/// [`DelimitedId`] (see [`validate`] for the requirements).
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(transparent)]
pub struct ArcDelimitedId<const N: usize> {
    inner: Arc<DelimitedIdRef<N>>,
}

impl<const N: usize> From<DelimitedId<N>> for ArcDelimitedId<N> {
    fn from(value: DelimitedId<N>) -> Self {
        Self {
            inner: Arc::from(value.inner),
        }
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

impl<const N: usize> Display for ArcDelimitedId<N> {
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

impl<const N: usize> AsRef<DelimitedIdRef<N>> for DelimitedId<N> {
    fn as_ref(&self) -> &DelimitedIdRef<N> {
        &self.inner
    }
}

impl<const N: usize> Deref for ArcDelimitedId<N> {
    type Target = DelimitedIdRef<N>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const N: usize> Borrow<DelimitedIdRef<N>> for ArcDelimitedId<N> {
    fn borrow(&self) -> &DelimitedIdRef<N> {
        &self.inner
    }
}

impl<const N: usize> AsRef<DelimitedIdRef<N>> for ArcDelimitedId<N> {
    fn as_ref(&self) -> &DelimitedIdRef<N> {
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

impl<const N: usize> Borrow<str> for ArcDelimitedId<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for ArcDelimitedId<N> {
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
impl<const N: usize> serde::Serialize for DelimitedId<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize> serde::Deserialize<'de> for DelimitedId<N> {
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
impl<const N: usize> serde::de::Visitor<'_> for DelimitedIdVisitor<N> {
    type Value = DelimitedId<N>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a delimited id string literal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(serde::de::Error::custom)
    }
}

/// A reference to a [`IdComponent`], akin to a `str`.
///
/// This is identical to `str` in every way, except that it has the invariant of being a valid
/// [`IdComponent`] (see [`validate`] for the requirements).
pub type IdComponentRef = DelimitedIdRef<1>;

impl Debug for IdComponentRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ident_component!(\"{}\")", &self.as_str())
    }
}

/// An owned component of a [`NamespacedId`] or [`OperationId`] (without delimiters).
///
/// This is identical to `Box<str>` in every way, except that it has the invariant of being a valid
/// [`IdComponent`] (see [`validate`] for the requirements).
pub type IdComponent = DelimitedId<1>;

impl Debug for IdComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Box(ident_component!(\"{}\"))", &self.as_str())
    }
}

pub type ArcIdComponent = ArcDelimitedId<1>;

impl Debug for ArcIdComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arc(ident_component!(\"{}\"))", &self.as_str())
    }
}

// id component concatenation

/// Implements addition for [`IdComponentRef`]s through concatenation.
///
/// This always works, as both sides are valid id components without separators.
impl<'a> Add<&'a IdComponentRef> for &IdComponentRef {
    type Output = IdComponent;
    fn add(self, rhs: &'a IdComponentRef) -> Self::Output {
        let combined = self.as_str().to_string() + rhs.as_str();
        let boxed = combined.into_boxed_str();
        IdComponent::from_box_unchecked(boxed)
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
    /// assert_eq!("namespace", ident!("namespace:path").namespace());
    /// ```
    #[must_use]
    pub const fn namespace(&self) -> &IdComponentRef {
        let namespace_len = self.namespace_len();
        let (namespace, _) = self.inner.split_at(namespace_len);
        IdComponentRef::from_str_unchecked(namespace)
    }

    /// Returns the portion of the id after the colon.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::ident;
    ///
    /// assert_eq!("path", ident!("namespace:path").path());
    /// ```
    #[must_use]
    pub const fn path(&self) -> &IdComponentRef {
        let namespace_len = self.namespace_len();
        let (_, id) = self.inner.split_at(namespace_len + 1);
        IdComponentRef::from_str_unchecked(id)
    }

    const fn namespace_len(&self) -> usize {
        let [namespace_len, _] = self.component_end_indicies();
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
    /// Creates a new [`NamespacedId`] from a separated `namespace` and `path`.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::{NamespacedId, ident_component};
    ///
    /// let id = NamespacedId::new(ident_component!("namespace"), ident_component!("path"));
    /// assert_eq!("namespace:path", id.as_str());
    /// ```
    #[must_use]
    pub fn new(namespace: &IdComponentRef, path: &IdComponentRef) -> Self {
        let string = format!("{namespace}:{path}");
        let boxed = Box::<str>::from(string);
        Self::from_box_unchecked(boxed)
    }
}

impl Debug for NamespacedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Box(ident!(\"{}\"))", &self.as_str())
    }
}

pub type ArcNamespacedId = ArcDelimitedId<2>;
impl Debug for ArcNamespacedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arc(ident!(\"{}\"))", &self.as_str())
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
    /// assert_eq!(ident!("namespace:path"), op_ident!("namespace:path:operation").namespaced_id());
    /// ```
    #[must_use]
    pub const fn namespaced_id(&self) -> &NamespacedIdRef {
        let [_, second_colon, _] = self.component_end_indicies();
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
    /// assert_eq!("namespace", op_ident!("namespace:path:operation").namespace());
    /// ```
    #[must_use]
    pub const fn namespace(&self) -> &IdComponentRef {
        self.namespaced_id().namespace()
    }

    /// Returns the second component of the id - the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::op_ident;
    ///
    /// assert_eq!("path", op_ident!("namespace:path:operation").path());
    /// ```
    #[must_use]
    pub const fn path(&self) -> &IdComponentRef {
        self.namespaced_id().path()
    }

    /// Returns the third component of the id - the operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::op_ident;
    ///
    /// assert_eq!("operation", op_ident!("namespace:path:operation").operation());
    /// ```
    #[must_use]
    pub const fn operation(&self) -> &IdComponentRef {
        let [_, second_colon, _] = self.component_end_indicies();
        let (_, operation) = self.inner.split_at(second_colon + 1);
        IdComponentRef::from_str_unchecked(operation)
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
    /// Creates a new [`OperationId`] from a separated `namespace`, `path`, and `operation`.
    ///
    /// # Examples
    ///
    /// ```
    /// use namespaced_id::{OperationId, ident_component};
    ///
    /// let id = OperationId::new(
    ///     ident_component!("namespace"),
    ///     ident_component!("path"),
    ///     ident_component!("operation"),
    /// );
    /// assert_eq!("namespace:path:operation", id.as_str());
    /// ```
    #[must_use]
    pub fn new(
        namespace: &IdComponentRef,
        path: &IdComponentRef,
        operation: &IdComponentRef,
    ) -> Self {
        let string = format!("{namespace}:{path}:{operation}");
        let boxed = Box::<str>::from(string);
        Self::from_box_unchecked(boxed)
    }
}

impl Debug for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Box(op_ident!(\"{}\"))", &self.as_str())
    }
}

pub type ArcOperationId = ArcDelimitedId<3>;

impl Debug for ArcOperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Arc(op_ident!(\"{}\"))", &self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use namespaced_id_macros::ident_component;

    use crate::{self as namespaced_id, DelimitedIdRef, NamespacedId};
    use crate::{NamespacedIdRef, ParseError, ident};

    #[test]
    fn roundtrip_check() {
        assert_eq!(
            Ok("namespace:path"),
            NamespacedIdRef::new("namespace:path").map(NamespacedIdRef::as_str)
        );
    }

    #[test]
    fn expected_namespace_error() {
        assert_eq!(
            Err(ParseError::UnexpectedComponentCount(0)),
            NamespacedIdRef::new("")
        );
    }

    #[test]
    fn expected_id_error() {
        assert_eq!(
            Err(ParseError::UnexpectedComponentCount(1)),
            NamespacedIdRef::new("a")
        );
    }

    #[test]
    fn too_many_separators_error() {
        assert_eq!(
            Err(ParseError::UnexpectedComponentCount(3)),
            NamespacedIdRef::new("a:b:c")
        );
    }

    #[test]
    fn unexpected_whitespace_error() {
        assert_eq!(
            Err(ParseError::UnexpectedCharacter(4)),
            NamespacedIdRef::new("name space:path")
        );
    }

    #[test]
    fn extract_namespace() {
        assert_eq!("namespace", ident!("namespace:path").namespace());
    }

    #[test]
    fn extract_id() {
        assert_eq!("path", ident!("namespace:path").path());
    }

    #[test]
    #[should_panic = "incorrect separator count"]
    fn namespace_panics_without_separator() {
        let _ = NamespacedIdRef::from_str_unchecked("no_separator").namespace();
    }

    #[test]
    #[should_panic = "incorrect separator count"]
    fn id_panics_without_separator() {
        let _ = NamespacedIdRef::from_str_unchecked("no_separator").path();
    }

    #[test]
    fn empty_namespace_and_id() {
        match NamespacedIdRef::new(":") {
            Ok(id) => {
                assert_eq!("", id.namespace());
                assert_eq!("", id.path());
            }
            Err(err) => {
                panic!("{err}");
            }
        }
    }

    #[test]
    fn owned_roundtrip() {
        assert_eq!(
            "namespace:path",
            ident!("namespace:path").to_owned().as_str()
        );
    }

    #[test]
    fn components_identity() {
        let id = ident_component!("ident");
        let [component] = id.components();

        assert_eq!(id, component);
        assert_eq!("ident", component);
    }

    #[test]
    fn concat() {
        let id = ident!("namespace:path");

        let [namespace, id] = id.components();
        let id = id + ident_component!("_suffixed");
        let id = NamespacedId::new(namespace, &id);

        assert_eq!("namespace:path_suffixed", id);
    }

    #[test]
    fn empty_id() {
        let id = DelimitedIdRef::<0>::new("")
            .expect("the empty string is a (and the only) valid DelimitedIdRef::<0>");

        // make sure some operations don't panic
        let [] = id.component_end_indicies();
        let [] = id.components();
        assert_eq!("", id.as_str());
    }

    #[test]
    fn empty_id_fails() {
        assert!(DelimitedIdRef::<0>::new("a").is_err());
        assert!(DelimitedIdRef::<0>::new(":").is_err());
        assert!(DelimitedIdRef::<0>::new(" ").is_err());
    }
}
