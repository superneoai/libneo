//! Supplies the shared string interface that GPUI imports.

use std::borrow::{Borrow, Cow};
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
enum Repr {
    Static(&'static str),
    Shared(Arc<str>),
}

/// An immutable string that can be cloned without copying its contents.
#[derive(Clone)]
pub struct SharedString(Repr);

impl SharedString {
    /// Creates a shared string without allocating.
    pub const fn new_static(value: &'static str) -> Self {
        Self(Repr::Static(value))
    }

    /// Creates a shared string from string-like input.
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(Repr::Shared(Arc::from(value.as_ref())))
    }

    /// Returns the string contents.
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Repr::Static(value) => value,
            Repr::Shared(value) => value,
        }
    }
}

impl Default for SharedString {
    fn default() -> Self {
        Self::new_static("")
    }
}

impl PartialEq for SharedString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for SharedString {}

impl PartialOrd for SharedString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SharedString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::hash::Hash for SharedString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(self.as_str(), state);
    }
}

impl std::ops::Deref for SharedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for SharedString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for SharedString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Debug for SharedString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(formatter)
    }
}

impl std::fmt::Display for SharedString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(formatter)
    }
}

impl JsonSchema for SharedString {
    fn inline_schema() -> bool {
        String::inline_schema()
    }

    fn schema_name() -> Cow<'static, str> {
        String::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}

impl Serialize for SharedString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SharedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from)
    }
}

impl PartialEq<String> for SharedString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<SharedString> for String {
    fn eq(&self, other: &SharedString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<str> for SharedString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for SharedString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl From<&SharedString> for SharedString {
    fn from(value: &SharedString) -> Self {
        value.clone()
    }
}

impl From<&str> for SharedString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<char> for SharedString {
    fn from(value: char) -> Self {
        Self::from(value.to_string())
    }
}

impl From<&mut str> for SharedString {
    fn from(value: &mut str) -> Self {
        Self::new(value)
    }
}

impl From<&String> for SharedString {
    fn from(value: &String) -> Self {
        Self::new(value)
    }
}

impl From<String> for SharedString {
    fn from(value: String) -> Self {
        Self(Repr::Shared(Arc::from(value)))
    }
}

impl From<Box<str>> for SharedString {
    fn from(value: Box<str>) -> Self {
        Self(Repr::Shared(Arc::from(value)))
    }
}

impl From<Arc<str>> for SharedString {
    fn from(value: Arc<str>) -> Self {
        Self(Repr::Shared(value))
    }
}

impl From<&Arc<str>> for SharedString {
    fn from(value: &Arc<str>) -> Self {
        Self(Repr::Shared(Arc::clone(value)))
    }
}

impl<'a> From<Cow<'a, str>> for SharedString {
    fn from(value: Cow<'a, str>) -> Self {
        match value {
            Cow::Borrowed(value) => Self::new(value),
            Cow::Owned(value) => Self::from(value),
        }
    }
}

impl From<SharedString> for Arc<str> {
    fn from(value: SharedString) -> Self {
        match value.0 {
            Repr::Static(value) => Arc::from(value),
            Repr::Shared(value) => value,
        }
    }
}

impl From<SharedString> for String {
    fn from(value: SharedString) -> Self {
        value.as_str().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::SharedString;
    use std::sync::Arc;

    #[test]
    fn static_and_shared_values_compare_equally() {
        let static_value = SharedString::new_static("libneo");
        let shared_value = SharedString::from(String::from("libneo"));

        assert_eq!(static_value, shared_value);
        assert_eq!(Arc::<str>::from(shared_value).as_ref(), "libneo");
    }
}
