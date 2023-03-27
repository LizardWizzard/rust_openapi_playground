use std::{fmt, str::FromStr};

use hex::FromHex;

/// Neon ID is a 128-bit random ID.
/// Used to represent various identifiers. Provides handy utility methods and impls.
///
/// NOTE: It (de)serializes as an array of hex bytes, so the string representation would look
/// like `[173,80,132,115,129,226,72,254,170,201,135,108,199,26,228,24]`.
///
/// Use `#[serde_as(as = "DisplayFromStr")]` to (de)serialize it as hex string instead: `ad50847381e248feaac9876cc71ae418`.
/// Check the `serde_with::serde_as` documentation for options for more complex types.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id([u8; 16]);

impl Id {
    pub fn get_from_buf(buf: &mut dyn bytes::Buf) -> Id {
        let mut arr = [0u8; 16];
        buf.copy_to_slice(&mut arr);
        Id::from(arr)
    }

    pub fn as_arr(&self) -> [u8; 16] {
        self.0
    }

    fn hex_encode(&self) -> String {
        static HEX: &[u8] = b"0123456789abcdef";

        let mut buf = vec![0u8; self.0.len() * 2];
        for (&b, chunk) in self.0.as_ref().iter().zip(buf.chunks_exact_mut(2)) {
            chunk[0] = HEX[((b >> 4) & 0xf) as usize];
            chunk[1] = HEX[(b & 0xf) as usize];
        }
        unsafe { String::from_utf8_unchecked(buf) }
    }
}

impl FromStr for Id {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Id, Self::Err> {
        Self::from_hex(s)
    }
}

// this is needed for pretty serialization and deserialization of Id's using serde integration with hex crate
impl FromHex for Id {
    type Error = hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut buf: [u8; 16] = [0u8; 16];
        hex::decode_to_slice(hex, &mut buf)?;
        Ok(Id(buf))
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 16]> for Id {
    fn from(b: [u8; 16]) -> Self {
        Id(b)
    }
}

impl From<Id> for u128 {
    fn from(id: Id) -> Self {
        u128::from_le_bytes(id.0)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex_encode())
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex_encode())
    }
}

macro_rules! id_newtype {
    ($t:ident) => {
        impl $t {
            pub fn get_from_buf(buf: &mut dyn bytes::Buf) -> $t {
                $t(Id::get_from_buf(buf))
            }

            pub fn as_arr(&self) -> [u8; 16] {
                self.0.as_arr()
            }

            pub const fn from_array(b: [u8; 16]) -> Self {
                $t(Id(b))
            }
        }

        impl FromStr for $t {
            type Err = hex::FromHexError;

            fn from_str(s: &str) -> Result<$t, Self::Err> {
                let value = Id::from_str(s)?;
                Ok($t(value))
            }
        }

        impl From<[u8; 16]> for $t {
            fn from(b: [u8; 16]) -> Self {
                $t(Id::from(b))
            }
        }

        impl FromHex for $t {
            type Error = hex::FromHexError;

            fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
                Ok($t(Id::from_hex(hex)?))
            }
        }

        impl AsRef<[u8]> for $t {
            fn as_ref(&self) -> &[u8] {
                &self.0 .0
            }
        }

        impl From<$t> for u128 {
            fn from(id: $t) -> Self {
                u128::from(id.0)
            }
        }

        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

/// Neon timeline IDs are different from PostgreSQL timeline
/// IDs. They serve a similar purpose though: they differentiate
/// between different "histories" of the same cluster.  However,
/// PostgreSQL timeline IDs are a bit cumbersome, because they are only
/// 32-bits wide, and they must be in ascending order in any given
/// timeline history.  Those limitations mean that we cannot generate a
/// new PostgreSQL timeline ID by just generating a random number. And
/// that in turn is problematic for the "pull/push" workflow, where you
/// have a local copy of a Neon repository, and you periodically sync
/// the local changes with a remote server. When you work "detached"
/// from the remote server, you cannot create a PostgreSQL timeline ID
/// that's guaranteed to be different from all existing timelines in
/// the remote server. For example, if two people are having a clone of
/// the repository on their laptops, and they both create a new branch
/// with different name. What timeline ID would they assign to their
/// branches? If they pick the same one, and later try to push the
/// branches to the same remote server, they will get mixed up.
///
/// To avoid those issues, Neon has its own concept of timelines that
/// is separate from PostgreSQL timelines, and doesn't have those
/// limitations. A Neon timeline is identified by a 128-bit ID, which
/// is usually printed out as a hex string.
///
/// NOTE: It (de)serializes as an array of hex bytes, so the string representation would look
/// like `[173,80,132,115,129,226,72,254,170,201,135,108,199,26,228,24]`.
/// See [`Id`] for alternative ways to serialize it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TimelineId(Id);

id_newtype!(TimelineId);

/// Neon Tenant Id represents identifiar of a particular tenant.
/// Is used for distinguishing requests and data belonging to different users.
///
/// NOTE: It (de)serializes as an array of hex bytes, so the string representation would look
/// like `[173,80,132,115,129,226,72,254,170,201,135,108,199,26,228,24]`.
/// See [`Id`] for alternative ways to serialize it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, poem_openapi::NewType)]
pub struct TenantId(Id);

id_newtype!(TenantId);

mod poem {
    use std::{borrow::Cow, str::FromStr};

    use poem_openapi::{
        registry::{MetaSchema, MetaSchemaRef},
        types::{ParseError, ParseResult},
    };

    use super::Id;

    impl poem_openapi::types::Type for Id {
        const IS_REQUIRED: bool = true;

        type RawValueType = Id;

        type RawElementValueType = Id;

        fn name() -> Cow<'static, str> {
            Cow::Borrowed("string(hex)")
        }

        fn schema_ref() -> MetaSchemaRef {
            let mut meta = MetaSchema::new_with_format("string", "hex");
            meta.description = Some("An Id type representing base for TimelineId and TenantId");
            MetaSchemaRef::Inline(Box::new(MetaSchema::new_with_format("string", "hex")))
        }

        fn as_raw_value(&self) -> Option<&Self::RawValueType> {
            Some(self)
        }

        fn raw_element_iter<'a>(
            &'a self,
        ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
            // Follows base64 impl (https://github.com/poem-web/poem/blob/master/poem-openapi/src/types/base64_type.rs)
            Box::new(self.as_raw_value().into_iter())
        }
    }

    impl poem_openapi::types::ParseFromJSON for Id {
        fn parse_from_json(
            value: Option<serde_json::Value>,
        ) -> poem_openapi::types::ParseResult<Self> {
            let value = match value {
                Some(value) => value,
                None => return ParseResult::Err(ParseError::expected_input()),
            };

            let value = match value.as_str() {
                Some(value) => value,
                None => return ParseResult::Err(ParseError::expected_type(value)),
            };

            Ok(Id::from_str(value)?)
        }
    }

    impl poem_openapi::types::ToJSON for Id {
        fn to_json(&self) -> Option<serde_json::Value> {
            Some(serde_json::Value::String(self.to_string()))
        }
    }

    impl poem_openapi::types::ParseFromParameter for Id {
        fn parse_from_parameter(value: &str) -> ParseResult<Self> {
            Ok(Id::from_str(value)?)
        }
    }

    impl poem_openapi::types::ToHeader for Id {
        fn to_header(&self) -> Option<poem::http::HeaderValue> {
            Some(
                poem::http::HeaderValue::from_str(&self.to_string())
                    .expect("id is a valid header value"),
            )
        }
    }

    #[poem::async_trait]
    impl poem_openapi::types::ParseFromMultipartField for Id {
        async fn parse_from_multipart(field: Option<poem::web::Field>) -> ParseResult<Self> {
            let field = match field {
                Some(field) => field,
                None => return ParseResult::Err(ParseError::expected_input()),
            };
            Ok(Id::from_str(&field.text().await?)?)
        }
    }
}
