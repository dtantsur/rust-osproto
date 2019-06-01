// Copyright 2019 Dmitry Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Common JSON structures and protocol bits.

use std::cmp::Ordering;
use std::fmt;
use std::iter::{DoubleEndedIterator, FusedIterator};
use std::str::FromStr;
use std::vec::IntoIter;

use serde::de::Error as DeserError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

/// A link to a resource.
#[derive(Clone, Debug, Deserialize)]
pub struct Link {
    #[serde(with = "url_serde")]
    pub href: Url,
    pub rel: String,
}

/// A reference to an ID and name.
#[derive(Clone, Debug, Deserialize)]
pub struct IdAndName {
    pub id: String,
    pub name: String,
}

/// A pair `X.Y` where `X` and `Y` can be converted to/from a string and `Y` is optional.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct XdotY<T>(pub T, pub T);

/// A single API version as returned by a version discovery endpoint.
#[derive(Clone, Debug, Deserialize)]
pub struct Version {
    #[serde(deserialize_with = "deser_version")]
    pub id: XdotY<u16>,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(deserialize_with = "empty_as_default", default)]
    pub status: Option<String>,
    #[serde(deserialize_with = "empty_as_default", default)]
    pub version: Option<XdotY<u16>>,
    #[serde(deserialize_with = "empty_as_default", default)]
    pub min_version: Option<XdotY<u16>>,
}

impl Version {
    /// Whether a version is considered stable according to its status.
    #[inline]
    pub fn is_stable(&self) -> bool {
        if let Some(ref status) = self.status {
            let upper = status.to_uppercase();
            upper == "STABLE" || upper == "CURRENT" || upper == "SUPPORTED"
        } else {
            true
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

/// A result of a version discovery endpoint.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Root {
    /// Multiple major versions.
    MultipleVersions { versions: Vec<Version> },
    /// Single major version.
    OneVersion { version: Version },
}

#[derive(Debug, Clone)]
enum IntoStableIterInner {
    Many(IntoIter<Version>),
    One(Option<Version>),
}

/// An iterator over stable versions.
#[derive(Debug)]
pub struct IntoStableIter(IntoStableIterInner);

impl Iterator for IntoStableIter {
    type Item = Version;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            IntoStableIterInner::Many(ref mut inner) => {
                for next in inner {
                    if next.is_stable() {
                        return Some(next);
                    }
                }

                None
            }
            IntoStableIterInner::One(ref mut opt) => opt.take(),
        }
    }
}

impl DoubleEndedIterator for IntoStableIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.0 {
            IntoStableIterInner::Many(ref mut inner) => {
                while let Some(next) = inner.next_back() {
                    if next.is_stable() {
                        return Some(next);
                    }
                }

                None
            }
            IntoStableIterInner::One(ref mut opt) => opt.take(),
        }
    }
}

impl FusedIterator for IntoStableIter where {}

impl Root {
    /// Sort versions from lowest to highest (using unstable sorting).
    #[inline]
    pub fn sort(&mut self) {
        if let Root::MultipleVersions {
            versions: ref mut vers,
        } = self
        {
            vers.sort_unstable();
        }
    }

    /// Create a `Root` sorted with `sort`.
    #[inline]
    pub fn into_sorted(mut self) -> Self {
        self.sort();
        self
    }

    /// Create an iterator over stable versions.
    pub fn into_stable_iter(self) -> IntoStableIter {
        match self {
            Root::MultipleVersions { versions: vers } => {
                IntoStableIter(IntoStableIterInner::Many(vers.into_iter()))
            }
            Root::OneVersion { version: ver } => {
                let stable = if ver.is_stable() { Some(ver) } else { None };
                IntoStableIter(IntoStableIterInner::One(stable))
            }
        }
    }
}

impl<T> fmt::Display for XdotY<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

impl<T> Serialize for XdotY<T>
where
    T: fmt::Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<T> FromStr for XdotY<T>
where
    T: FromStr + Default,
    T::Err: fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<XdotY<T>, String> {
        let mut parts = s.split('.');

        if let Some(x_part) = parts.next() {
            let x = x_part
                .parse()
                .map_err(|err| format!("cannot parse the first component: {}", err))?;

            let y = if let Some(y_part) = parts.next() {
                y_part
                    .parse()
                    .map_err(|err| format!("cannot parse the second component: {}", err))?
            } else {
                T::default()
            };

            if parts.next().is_some() {
                Err(format!("expected X.Y, got {}", s))
            } else {
                Ok(XdotY(x, y))
            }
        } else {
            Err(format!("expected X.Y, got {}", s))
        }
    }
}

impl<'de, T> Deserialize<'de> for XdotY<T>
where
    T: FromStr + Default,
    T::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<XdotY<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: &str = Deserialize::deserialize(deserializer)?;
        XdotY::from_str(value).map_err(D::Error::custom)
    }
}

fn deser_version<'de, D, T>(des: D) -> Result<XdotY<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default,
    T::Err: fmt::Display,
{
    let value: &str = Deserialize::deserialize(des)?;
    if value.is_empty() {
        return Err(D::Error::custom("Empty version ID"));
    }

    let version_part = if value.starts_with('v') {
        &value[1..]
    } else {
        &value
    };

    XdotY::from_str(version_part).map_err(D::Error::custom)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ValueOrString<'s, T> {
    Value(T),
    String(&'s str),
}

/// Deserialize a value where empty string is replaced by `Default` value.
pub fn empty_as_default<'de, D, T>(des: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    match ValueOrString::deserialize(des)? {
        ValueOrString::Value(val) => Ok(val),
        ValueOrString::String(val) => {
            if val == "" {
                Ok(T::default())
            } else {
                Err(DeserError::custom("Unexpected non-empty string"))
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};
    use serde_json;

    use super::{empty_as_default, Root, Version, XdotY};

    pub fn compare<T: Serialize>(sample: &str, value: T) {
        let converted: serde_json::Value = serde_json::from_str(sample).unwrap();
        let result = serde_json::to_value(value).unwrap();
        assert_eq!(result, converted);
    }

    #[derive(Debug, Deserialize)]
    struct Custom(bool);

    #[derive(Debug, Deserialize)]
    struct EmptyAsDefault {
        #[serde(deserialize_with = "empty_as_default")]
        number: u8,
        #[serde(deserialize_with = "empty_as_default")]
        vec: Vec<String>,
        #[serde(deserialize_with = "empty_as_default")]
        opt: Option<Custom>,
    }

    #[test]
    fn test_empty_as_default_with_values() {
        let s = "{\"number\": 42, \"vec\": [\"value\"], \"opt\": true}";
        let r: EmptyAsDefault = serde_json::from_str(s).unwrap();
        assert_eq!(r.number, 42);
        assert_eq!(r.vec, vec!["value".to_string()]);
        assert!(r.opt.unwrap().0);
    }

    #[test]
    fn test_empty_as_default_with_empty_string() {
        let s = "{\"number\": \"\", \"vec\": \"\", \"opt\": \"\"}";
        let r: EmptyAsDefault = serde_json::from_str(s).unwrap();
        assert_eq!(r.number, 0);
        assert!(r.vec.is_empty());
        assert!(r.opt.is_none());
    }

    #[test]
    fn test_xdoty_display() {
        let xy = XdotY(1, 2);
        let s = format!("{}", xy);
        assert_eq!(s, "1.2");
    }

    #[test]
    fn test_xdoty_from_str() {
        let xy: XdotY<u8> = XdotY::from_str("1.2").unwrap();
        assert_eq!(xy.0, 1);
        assert_eq!(xy.1, 2);
    }

    #[test]
    fn test_xdoty_from_str_no_y() {
        let xy: XdotY<u8> = XdotY::from_str("1").unwrap();
        assert_eq!(xy.0, 1);
        assert_eq!(xy.1, 0);
    }

    #[test]
    fn test_xdoty_from_str_failure() {
        for s in &["foo", "1.foo", "foo.2", "1.2.3"] {
            let res: Result<XdotY<u8>, _> = XdotY::from_str(s);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_xdoty_serde_serialize() {
        let xy = XdotY(2u8, 27);
        let ser = serde_json::to_string(&xy).unwrap();
        assert_eq!(&ser, "\"2.27\"");
    }

    #[test]
    fn test_xdoty_serde_deserialize() {
        let xy: XdotY<u8> = serde_json::from_str("\"2.27\"").unwrap();
        assert_eq!(xy.0, 2);
        assert_eq!(xy.1, 27);
    }

    #[test]
    fn test_version_current_is_stable() {
        let stable = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("CURRENT".to_string()),
            version: None,
            min_version: None,
        };
        assert!(stable.is_stable());
    }

    #[test]
    fn test_version_stable_is_stable() {
        let stable = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("Stable".to_string()),
            version: None,
            min_version: None,
        };
        assert!(stable.is_stable());
    }

    #[test]
    fn test_version_supported_is_stable() {
        let stable = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("supported".to_string()),
            version: None,
            min_version: None,
        };
        assert!(stable.is_stable());
    }

    #[test]
    fn test_version_no_status_is_stable() {
        let stable = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: None,
            version: None,
            min_version: None,
        };
        assert!(stable.is_stable());
    }

    #[test]
    fn test_version_deprecated_is_not_stable() {
        let unstable = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("DEPRECATED".to_string()),
            version: None,
            min_version: None,
        };
        assert!(!unstable.is_stable());
    }

    #[test]
    fn test_root_sort() {
        let vers: Vec<_> = [3, 1, 2]
            .into_iter()
            .map(|idx| Version {
                id: XdotY(*idx, 0),
                links: Vec::new(),
                status: None,
                version: None,
                min_version: None,
            })
            .collect();
        let mut root = Root::MultipleVersions { versions: vers };
        root.sort();
        if let Root::MultipleVersions { versions: res } = root {
            let idx = res.into_iter().map(|v| v.id.0).collect::<Vec<_>>();
            assert_eq!(idx, vec![1, 2, 3]);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_root_sort_one() {
        let ver = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("supported".to_string()),
            version: None,
            min_version: None,
        };
        let mut root = Root::OneVersion { version: ver };
        root.sort();
        if let Root::OneVersion { version: res } = root {
            assert_eq!(res.id.0, 2);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_root_into_sorted() {
        let vers: Vec<_> = [3, 1, 2]
            .into_iter()
            .map(|idx| Version {
                id: XdotY(*idx, 0),
                links: Vec::new(),
                status: None,
                version: None,
                min_version: None,
            })
            .collect();
        let mut root = Root::MultipleVersions { versions: vers };
        root = root.into_sorted();
        if let Root::MultipleVersions { versions: res } = root {
            let idx = res.into_iter().map(|v| v.id.0).collect::<Vec<_>>();
            assert_eq!(idx, vec![1, 2, 3]);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_root_into_stable_iter() {
        let vers: Vec<_> = [3, 1, 2]
            .into_iter()
            .map(|idx| Version {
                id: XdotY(*idx, 0),
                links: Vec::new(),
                status: Some(if *idx > 1 { "CURRENT" } else { "DEPRECATED" }.to_string()),
                version: None,
                min_version: None,
            })
            .collect();
        let root = Root::MultipleVersions { versions: vers };
        let idx = root
            .into_stable_iter()
            .map(|ver| ver.id.0)
            .collect::<Vec<_>>();
        assert_eq!(idx, vec![3, 2]);
    }

    #[test]
    fn test_root_into_stable_iter_reverse() {
        let vers: Vec<_> = [3, 1, 2]
            .into_iter()
            .map(|idx| Version {
                id: XdotY(*idx, 0),
                links: Vec::new(),
                status: Some(if *idx > 1 { "CURRENT" } else { "DEPRECATED" }.to_string()),
                version: None,
                min_version: None,
            })
            .collect();
        let root = Root::MultipleVersions { versions: vers };
        let mut idx = root.into_stable_iter().map(|ver| ver.id.0);
        assert_eq!(idx.next_back(), Some(2));
        assert_eq!(idx.next_back(), Some(3));
        assert!(idx.next_back().is_none());
        assert!(idx.next().is_none());
    }

    #[test]
    fn test_root_into_stable_iter_one() {
        let ver = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("supported".to_string()),
            version: None,
            min_version: None,
        };
        let root = Root::OneVersion { version: ver };
        let idx = root
            .into_stable_iter()
            .map(|ver| ver.id.0)
            .collect::<Vec<_>>();
        assert_eq!(idx, vec![2]);
    }

    #[test]
    fn test_root_into_stable_iter_one_unstable() {
        let ver = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("deprecated".to_string()),
            version: None,
            min_version: None,
        };
        let root = Root::OneVersion { version: ver };
        let mut idx = root.into_stable_iter().map(|ver| ver.id.0);
        assert!(idx.next().is_none());
    }

    #[test]
    fn test_root_into_stable_iter_one_reverse() {
        let ver = Version {
            id: XdotY(2, 0),
            links: Vec::new(),
            status: Some("supported".to_string()),
            version: None,
            min_version: None,
        };
        let root = Root::OneVersion { version: ver };
        let mut idx = root.into_stable_iter().map(|ver| ver.id.0);
        assert_eq!(idx.next_back(), Some(2));
        assert!(idx.next_back().is_none());
    }
}
