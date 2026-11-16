//! Turning captured URL parameters into the handler's first argument.

use super::parameters::UrlParams;
use std::{error::Error, fmt};

/// Types a handler can take as its first argument, built from the captured URL params.
///
/// Implemented for [`UrlParams`] (no conversion) and, with the `serde` feature, for
/// [`Params<T>`](crate::Params). If conversion fails the handler is not called and the
/// router answers `400 Bad Request` through the error handler.
pub trait FromUrlParams: Sized {
    /// Build `Self` from the captured params.
    ///
    /// # Errors
    ///
    /// Returns a [`ParamsRejection`] when the params can't be converted.
    fn from_url_params(params: UrlParams) -> Result<Self, ParamsRejection>;
}

impl FromUrlParams for UrlParams {
    fn from_url_params(params: UrlParams) -> Result<Self, ParamsRejection> {
        Ok(params)
    }
}

/// Why URL params couldn't be converted for a handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamsRejection {
    message: String,
}

impl ParamsRejection {
    /// Create a rejection with a human-readable reason.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParamsRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid URL parameters: {}", self.message)
    }
}

impl Error for ParamsRejection {}

#[cfg(feature = "serde")]
pub use self::serde_params::Params;

#[cfg(feature = "serde")]
mod serde_params {
    use super::{FromUrlParams, ParamsRejection, UrlParams};
    use serde::de::{
        self, DeserializeOwned, Deserializer, IntoDeserializer, Visitor,
        value::{Error as DeError, MapDeserializer, SeqDeserializer},
    };

    /// Typed URL params, deserialized with serde.
    ///
    /// - a struct (or map) reads params by name: `Params<UserPath>`
    /// - a tuple (or `Vec`) reads them in path order: `Params<(String, u32)>`
    /// - a single value reads the only param: `Params<u64>`
    ///
    /// Values are parsed from strings, so numbers, `bool`, `char`, `Option` and unit-variant
    /// enums work.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Params<T>(pub T);

    impl<T: DeserializeOwned> FromUrlParams for Params<T> {
        fn from_url_params(params: UrlParams) -> Result<Self, ParamsRejection> {
            let pairs = params.into_pairs();
            T::deserialize(ParamsDeserializer { pairs })
                .map(Params)
                .map_err(|e| ParamsRejection::new(e.to_string()))
        }
    }

    /// Deserializes the whole param list as a map, a sequence or a single value.
    struct ParamsDeserializer {
        pairs: Vec<(String, String)>,
    }

    impl ParamsDeserializer {
        fn single(self) -> Result<ValueDeserializer, DeError> {
            let len = self.pairs.len();
            let mut pairs = self.pairs.into_iter();
            match (pairs.next(), len) {
                (Some((_, value)), 1) => Ok(ValueDeserializer(value)),
                _ => Err(de::Error::custom(format!(
                    "expected exactly 1 URL parameter, found {len}"
                ))),
            }
        }

        fn values(self) -> SeqDeserializer<std::vec::IntoIter<ValueDeserializer>, DeError> {
            let values: Vec<_> = self
                .pairs
                .into_iter()
                .map(|(_, v)| ValueDeserializer(v))
                .collect();
            SeqDeserializer::new(values.into_iter())
        }
    }

    macro_rules! single_value {
        ($($method:ident)*) => {$(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                self.single()?.$method(visitor)
            }
        )*};
    }

    impl<'de> Deserializer<'de> for ParamsDeserializer {
        type Error = DeError;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            self.deserialize_map(visitor)
        }

        fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            let pairs = self
                .pairs
                .into_iter()
                .map(|(k, v)| (k, ValueDeserializer(v)));
            visitor.visit_map(MapDeserializer::new(pairs))
        }

        fn deserialize_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.deserialize_map(visitor)
        }

        fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.visit_seq(self.values())
        }

        fn deserialize_tuple<V: Visitor<'de>>(
            self,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            if self.pairs.len() != len {
                return Err(de::Error::invalid_length(self.pairs.len(), &visitor));
            }
            self.deserialize_seq(visitor)
        }

        fn deserialize_tuple_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.deserialize_tuple(len, visitor)
        }

        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_enum<V: Visitor<'de>>(
            self,
            name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.single()?.deserialize_enum(name, variants, visitor)
        }

        single_value! {
            deserialize_bool deserialize_i8 deserialize_i16 deserialize_i32 deserialize_i64
            deserialize_i128 deserialize_u8 deserialize_u16 deserialize_u32 deserialize_u64
            deserialize_u128 deserialize_f32 deserialize_f64 deserialize_char deserialize_str
            deserialize_string deserialize_bytes deserialize_byte_buf deserialize_option
            deserialize_unit deserialize_identifier deserialize_ignored_any
        }

        fn deserialize_unit_struct<V: Visitor<'de>>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.single()?.deserialize_unit_struct(name, visitor)
        }
    }

    /// One param value, parsed from its string form on demand.
    struct ValueDeserializer(String);

    impl IntoDeserializer<'_, DeError> for ValueDeserializer {
        type Deserializer = Self;

        fn into_deserializer(self) -> Self {
            self
        }
    }

    macro_rules! parse_value {
        ($($method:ident => $visit:ident : $ty:ty),* $(,)?) => {$(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                match self.0.parse::<$ty>() {
                    Ok(value) => visitor.$visit(value),
                    Err(_) => Err(de::Error::invalid_value(
                        de::Unexpected::Str(&self.0),
                        &stringify!($ty),
                    )),
                }
            }
        )*};
    }

    impl<'de> Deserializer<'de> for ValueDeserializer {
        type Error = DeError;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.visit_string(self.0)
        }

        parse_value! {
            deserialize_bool => visit_bool: bool,
            deserialize_i8 => visit_i8: i8,
            deserialize_i16 => visit_i16: i16,
            deserialize_i32 => visit_i32: i32,
            deserialize_i64 => visit_i64: i64,
            deserialize_i128 => visit_i128: i128,
            deserialize_u8 => visit_u8: u8,
            deserialize_u16 => visit_u16: u16,
            deserialize_u32 => visit_u32: u32,
            deserialize_u64 => visit_u64: u64,
            deserialize_u128 => visit_u128: u128,
            deserialize_f32 => visit_f32: f32,
            deserialize_f64 => visit_f64: f64,
            deserialize_char => visit_char: char,
        }

        fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.visit_some(self)
        }

        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_newtype_struct(self)
        }

        fn deserialize_enum<V: Visitor<'de>>(
            self,
            _name: &'static str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_enum(self.0.into_deserializer())
        }

        serde::forward_to_deserialize_any! {
            str string bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
            identifier ignored_any
        }
    }
}
