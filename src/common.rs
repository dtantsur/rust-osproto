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

use reqwest::Url;
use serde::de::{DeserializeOwned, Error as DeserError};
use serde::{Deserialize, Deserializer};

/// A link to a resource.
#[derive(Clone, Debug, Deserialize)]
pub struct Link {
    #[serde(deserialize_with = "deser_url")]
    pub href: Url,
    pub rel: String,
}

/// A reference to an ID and name.
#[derive(Clone, Debug, Deserialize)]
pub struct IdAndName {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ValueOrString<T> {
    Value(T),
    String(String),
}

/// Deserialize value where empty string equals None.
pub fn empty_as_none<'de, D, T>(des: D) -> ::std::result::Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = ValueOrString::deserialize(des)?;
    match value {
        ValueOrString::Value(val) => Ok(Some(val)),
        ValueOrString::String(val) => {
            if val == "" {
                Ok(None)
            } else {
                Err(DeserError::custom("Unexpected string"))
            }
        }
    }
}

/// Deserialize a URL.
pub fn deser_url<'de, D>(des: D) -> ::std::result::Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    Url::parse(&String::deserialize(des)?).map_err(DeserError::custom)
}

#[cfg(test)]
mod test {
    use reqwest::Url;
    use serde::Deserialize;
    use serde_json;

    use super::{deser_url, empty_as_none};

    #[derive(Debug, Deserialize)]
    struct EmptyAsNone {
        #[serde(deserialize_with = "empty_as_none")]
        number: Option<u8>,
        #[serde(deserialize_with = "empty_as_none")]
        vec: Option<Vec<String>>,
    }

    #[derive(Debug, Deserialize)]
    struct DeserUrl {
        #[serde(deserialize_with = "deser_url")]
        url: Url,
    }

    #[test]
    fn test_empty_as_none_with_values() {
        let s = "{\"number\": 42, \"vec\": [\"value\"]}";
        let r: EmptyAsNone = serde_json::from_str(s).unwrap();
        assert_eq!(r.number.unwrap(), 42);
        assert_eq!(r.vec.unwrap(), vec!["value".to_string()]);
    }

    #[test]
    fn test_empty_as_none_with_empty_string() {
        let s = "{\"number\": \"\", \"vec\": \"\"}";
        let r: EmptyAsNone = serde_json::from_str(s).unwrap();
        assert!(r.number.is_none());
        assert!(r.vec.is_none());
    }

    #[test]
    fn test_deser_url() {
        let s = "{\"url\": \"https://127.0.0.1/path\"}";
        let r: DeserUrl = serde_json::from_str(s).unwrap();
        assert_eq!(r.url.as_str(), "https://127.0.0.1/path");
    }
}
