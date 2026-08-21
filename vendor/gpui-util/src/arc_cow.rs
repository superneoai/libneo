//! Borrowed-or-shared values.

use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// A value that is either borrowed or atomically reference counted.
pub enum ArcCow<'a, T: ?Sized> {
    /// A borrowed value.
    Borrowed(&'a T),
    /// A shared owned value.
    Owned(Arc<T>),
}

impl<T: ?Sized + PartialEq> PartialEq for ArcCow<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T: ?Sized + Eq> Eq for ArcCow<'_, T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for ArcCow<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T: ?Sized + Ord> Ord for ArcCow<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized + Hash> Hash for ArcCow<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<T: ?Sized> Clone for ArcCow<'_, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(value) => Self::Borrowed(value),
            Self::Owned(value) => Self::Owned(Arc::clone(value)),
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for ArcCow<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

impl<T: ?Sized> From<Arc<T>> for ArcCow<'_, T> {
    fn from(value: Arc<T>) -> Self {
        Self::Owned(value)
    }
}

impl<T: ?Sized> From<&Arc<T>> for ArcCow<'_, T> {
    fn from(value: &Arc<T>) -> Self {
        Self::Owned(Arc::clone(value))
    }
}

impl From<String> for ArcCow<'_, str> {
    fn from(value: String) -> Self {
        Self::Owned(Arc::from(value))
    }
}

impl From<&String> for ArcCow<'_, str> {
    fn from(value: &String) -> Self {
        Self::Owned(Arc::from(value.as_str()))
    }
}

impl<'a> From<Cow<'a, str>> for ArcCow<'a, str> {
    fn from(value: Cow<'a, str>) -> Self {
        match value {
            Cow::Borrowed(value) => Self::Borrowed(value),
            Cow::Owned(value) => Self::Owned(Arc::from(value)),
        }
    }
}

impl<T> From<Vec<T>> for ArcCow<'_, [T]> {
    fn from(value: Vec<T>) -> Self {
        Self::Owned(Arc::from(value))
    }
}

impl<'a> From<&'a str> for ArcCow<'a, [u8]> {
    fn from(value: &'a str) -> Self {
        Self::Borrowed(value.as_bytes())
    }
}

impl<T: ?Sized + ToOwned> Borrow<T> for ArcCow<'_, T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> std::ops::Deref for ArcCow<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> AsRef<T> for ArcCow<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }
}

impl<T: ?Sized + Debug> Debug for ArcCow<'_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(formatter)
    }
}
