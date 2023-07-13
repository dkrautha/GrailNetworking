use crate::{
    util::{read_bytes, read_string, write_bytes, write_string},
    XbfPrimitiveMetadata, XbfTypeUpcast,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

/// A primitive type as defined by the XBF specification.
///
/// Each XBF primitive maps to the corresponding Rust type, with the exception of 256 bit numbers,
/// which are represented as a `[u64; 4]`.
#[derive(Debug, Clone, PartialEq)]
pub enum XbfPrimitive {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256([u64; 4]),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    I256([u64; 4]),
    F32(f32),
    F64(f64),
    Bytes(Vec<u8>),
    String(String),
}

impl XbfPrimitive {
    /// Serialize a primitive type as defined by the XBF specification.
    ///
    /// This function **does not** write out the metadata of the type. If you want to write out the
    /// metadata, convert this type to a [`XbfPrimitiveMetadata`] and call
    /// [`XbfPrimitiveMetadata::serialize_primitive_metadata`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitive;
    ///
    /// let primitive = XbfPrimitive::U64(420);
    /// let mut writer = Vec::new();
    /// primitive.serialize_primitive_type(&mut writer).unwrap();
    ///
    /// assert_eq!(writer, 420u64.to_le_bytes());
    /// ```
    pub fn serialize_primitive_type(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            XbfPrimitive::Bool(x) => writer.write_u8(u8::from(*x)),
            XbfPrimitive::U8(x) => writer.write_u8(*x),
            XbfPrimitive::U16(x) => writer.write_u16::<LittleEndian>(*x),
            XbfPrimitive::U32(x) => writer.write_u32::<LittleEndian>(*x),
            XbfPrimitive::U64(x) => writer.write_u64::<LittleEndian>(*x),
            XbfPrimitive::U128(x) => writer.write_u128::<LittleEndian>(*x),
            XbfPrimitive::U256(x) => x
                .iter()
                .try_for_each(|x| writer.write_u64::<LittleEndian>(*x)),
            XbfPrimitive::I8(x) => writer.write_i8(*x),
            XbfPrimitive::I16(x) => writer.write_i16::<LittleEndian>(*x),
            XbfPrimitive::I32(x) => writer.write_i32::<LittleEndian>(*x),
            XbfPrimitive::I64(x) => writer.write_i64::<LittleEndian>(*x),
            XbfPrimitive::I128(x) => writer.write_i128::<LittleEndian>(*x),
            XbfPrimitive::I256(x) => x
                .iter()
                .try_for_each(|x| writer.write_u64::<LittleEndian>(*x)),
            XbfPrimitive::F32(x) => writer.write_f32::<LittleEndian>(*x),
            XbfPrimitive::F64(x) => writer.write_f64::<LittleEndian>(*x),
            XbfPrimitive::Bytes(x) => write_bytes(x, writer),
            XbfPrimitive::String(x) => write_string(x, writer),
        }
    }

    /// Deserialize a primitive type as defined by the XBF specification.
    ///
    /// This function **does not** read the metadata of the type from the reader. It is expected
    /// that to call this function the metadata for a type is already known, be that from reading
    /// it from the reader with [`deserialize_base_metadata`](crate::XbfMetadata::deserialize_base_metadata)
    /// or having it in some other manner.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let metadata = XbfPrimitiveMetadata::I32;
    /// let mut reader = std::io::Cursor::new(69i32.to_le_bytes());
    ///
    /// let primitive = XbfPrimitive::deserialize_primitive_type(&metadata, &mut reader).unwrap();
    /// ```
    pub fn deserialize_primitive_type(
        primitive_metadata: &XbfPrimitiveMetadata,
        reader: &mut impl Read,
    ) -> io::Result<XbfPrimitive> {
        match primitive_metadata {
            XbfPrimitiveMetadata::Bool => reader.read_u8().map(|x| XbfPrimitive::Bool(x != 0)),
            XbfPrimitiveMetadata::U8 => reader.read_u8().map(XbfPrimitive::U8),
            XbfPrimitiveMetadata::U16 => reader.read_u16::<LittleEndian>().map(XbfPrimitive::U16),
            XbfPrimitiveMetadata::U32 => reader.read_u32::<LittleEndian>().map(XbfPrimitive::U32),
            XbfPrimitiveMetadata::U64 => reader.read_u64::<LittleEndian>().map(XbfPrimitive::U64),
            XbfPrimitiveMetadata::U128 => {
                reader.read_u128::<LittleEndian>().map(XbfPrimitive::U128)
            }
            XbfPrimitiveMetadata::U256 => {
                let mut data = [0; 4];
                for i in &mut data {
                    *i = reader.read_u64::<LittleEndian>()?
                }
                Ok(XbfPrimitive::U256(data))
            }
            XbfPrimitiveMetadata::I8 => reader.read_i8().map(XbfPrimitive::I8),
            XbfPrimitiveMetadata::I16 => reader.read_i16::<LittleEndian>().map(XbfPrimitive::I16),
            XbfPrimitiveMetadata::I32 => reader.read_i32::<LittleEndian>().map(XbfPrimitive::I32),
            XbfPrimitiveMetadata::I64 => reader.read_i64::<LittleEndian>().map(XbfPrimitive::I64),
            XbfPrimitiveMetadata::I128 => {
                reader.read_i128::<LittleEndian>().map(XbfPrimitive::I128)
            }
            XbfPrimitiveMetadata::I256 => {
                let mut data = [0; 4];
                for i in &mut data {
                    *i = reader.read_u64::<LittleEndian>()?
                }
                Ok(XbfPrimitive::I256(data))
            }
            XbfPrimitiveMetadata::F32 => reader.read_f32::<LittleEndian>().map(XbfPrimitive::F32),
            XbfPrimitiveMetadata::F64 => reader.read_f64::<LittleEndian>().map(XbfPrimitive::F64),
            XbfPrimitiveMetadata::Bytes => read_bytes(reader).map(XbfPrimitive::Bytes),
            XbfPrimitiveMetadata::String => read_string(reader).map(XbfPrimitive::String),
        }
    }

    /// Get the metadata for this primitive type.
    ///
    /// Primitive metadata is only a discriminant value, so an owned copy of the metadata is
    /// returned as doing so is inexpensive.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let primitive = XbfPrimitive::I32(55);
    /// let metadata = primitive.get_metadata();
    ///
    /// assert_eq!(metadata, XbfPrimitiveMetadata::I32);
    /// ```
    pub fn get_metadata(&self) -> XbfPrimitiveMetadata {
        XbfPrimitiveMetadata::from(self)
    }
}

impl XbfTypeUpcast for XbfPrimitive {}

// TODO: Seal this trait so that nobody can implement it
// TODO: Are the names of the provided methods correct?
// TODO: Should both methods be in one trait?
//
/// A trait for converting native Rust types to [`XbfPrimitive`] types.
///
/// This trait is implemented for all of the supported types that have an XBF equivalent.
///
/// You should not implement this trait yourself.
pub trait NativeToXbfPrimitive: Into<XbfPrimitive>
where
    XbfPrimitive: for<'a> From<&'a Self>,
{
    /// Convert a native Rust type to a [`XbfPrimitive`] type.
    ///
    /// Calling this method may result in a clone if the type is [`XbfPrimitive::Bytes`]
    /// or [`XbfPrimitive::String`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::NativeToXbfPrimitive;
    ///
    /// let native = 69i32;
    /// let primitive = native.to_xbf_primitive();
    ///
    /// assert_eq!(primitive, XbfPrimitive::I32(69));
    /// ```
    fn to_xbf_primitive(&self) -> XbfPrimitive {
        self.into()
    }

    /// Convert a native Rust type to a [`XbfPrimitive`] type, consuming `self`.
    ///
    /// Calling this method should not result in a clone.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::NativeToXbfPrimitive;
    ///
    /// let native = 69i32;
    /// let primitive = native.into_xbf_primitive();
    ///
    /// assert_eq!(primitive, XbfPrimitive::I32(69));
    /// ```
    fn into_xbf_primitive(self) -> XbfPrimitive {
        self.into()
    }
}

macro_rules! impl_NativeToXbfPrimitive {
    ($ty:ty, $xbf_type:tt) => {
        impl From<$ty> for XbfPrimitive {
            fn from(x: $ty) -> Self {
                XbfPrimitive::$xbf_type(x)
            }
        }

        impl From<&$ty> for XbfPrimitive {
            fn from(x: &$ty) -> Self {
                XbfPrimitive::$xbf_type(x.clone())
            }
        }

        impl NativeToXbfPrimitive for $ty {}
    };
}

impl_NativeToXbfPrimitive!(bool, Bool);
impl_NativeToXbfPrimitive!(u8, U8);
impl_NativeToXbfPrimitive!(u16, U16);
impl_NativeToXbfPrimitive!(u32, U32);
impl_NativeToXbfPrimitive!(u64, U64);
impl_NativeToXbfPrimitive!(u128, U128);
impl_NativeToXbfPrimitive!(i8, I8);
impl_NativeToXbfPrimitive!(i16, I16);
impl_NativeToXbfPrimitive!(i32, I32);
impl_NativeToXbfPrimitive!(i64, I64);
impl_NativeToXbfPrimitive!(i128, I128);
impl_NativeToXbfPrimitive!(f32, F32);
impl_NativeToXbfPrimitive!(f64, F64);
impl_NativeToXbfPrimitive!(Vec<u8>, Bytes);
impl_NativeToXbfPrimitive!(String, String);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{XbfMetadata, XbfType};
    use std::io::Cursor;

    mod serde {
        use super::*;

        macro_rules! serde_primitive_test {
            ($xbf_type:tt, $test_num:expr) => {
                serde_primitive_test!($xbf_type, $test_num, $test_num);
            };
            ($xbf_type:tt, $test_val:expr, $to_bytes:expr) => {
                let primitive = $test_val.to_xbf_primitive();
                let mut writer = Vec::new();

                primitive.serialize_primitive_type(&mut writer).unwrap();

                let expected = $to_bytes.to_le_bytes();
                assert_eq!(writer, expected);

                let mut reader = Cursor::new(writer);

                let metadata = XbfMetadata::Primitive(XbfPrimitiveMetadata::$xbf_type);
                let expected = XbfType::Primitive($test_val.into_xbf_primitive());

                let primitive = XbfType::deserialize_base_type(&metadata, &mut reader).unwrap();
                assert_eq!(primitive, expected);
            };
        }

        #[test]
        fn bool_works() {
            serde_primitive_test!(Bool, true, 1u8);
            serde_primitive_test!(Bool, false, 0u8);
        }

        #[test]
        fn unsigned_nums_works() {
            serde_primitive_test!(U8, 42u8);
            serde_primitive_test!(U16, 420u16);
            serde_primitive_test!(U32, 100_000u32);
            serde_primitive_test!(U64, 100_000_000u64);
            serde_primitive_test!(U128, 18_446_744_073_709_551_617u128);
        }

        #[test]
        fn u256_works() {
            const TEST_NUM: [u64; 4] = [1, 2, 3, 4];
            let primitive = XbfPrimitive::U256(TEST_NUM);
            let mut writer = Vec::new();

            primitive.serialize_primitive_type(&mut writer).unwrap();

            let expected = TEST_NUM
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>();
            assert_eq!(writer, expected);

            let mut reader = Cursor::new(writer);
            let deserialized = XbfType::deserialize_base_type(
                &XbfMetadata::Primitive(XbfPrimitiveMetadata::U256),
                &mut reader,
            )
            .unwrap();
            assert_eq!(deserialized, primitive.to_base_type());
        }

        #[test]
        fn signed_nums_works() {
            serde_primitive_test!(I8, 42i8);
            serde_primitive_test!(I16, 420i16);
            serde_primitive_test!(I32, 100_000i32);
            serde_primitive_test!(I64, 100_000_000i64);
            serde_primitive_test!(I128, 18_446_744_073_709_551_617i128);
        }

        #[test]
        fn i256_works() {
            const TEST_NUM: [u64; 4] = [1, 2, 3, 4];
            let primitive = XbfPrimitive::I256(TEST_NUM);
            let mut writer = Vec::new();

            primitive.serialize_primitive_type(&mut writer).unwrap();

            let expected = TEST_NUM
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>();
            assert_eq!(writer, expected);

            let mut reader = Cursor::new(writer);
            let deserialized = XbfType::deserialize_base_type(
                &XbfMetadata::Primitive(XbfPrimitiveMetadata::I256),
                &mut reader,
            )
            .unwrap();
            assert_eq!(deserialized, primitive.to_base_type());
        }

        #[test]
        fn floating_point_works() {
            serde_primitive_test!(F32, 69.0f32);
            serde_primitive_test!(F64, 69.0f64);
        }

        #[test]
        fn string_works() {
            let test_string = "hello world".to_string();
            let primitive = XbfPrimitive::String(test_string.clone());
            let mut writer = vec![];

            primitive.serialize_primitive_type(&mut writer).unwrap();

            let mut expected_writer = vec![];
            expected_writer.extend_from_slice(&(test_string.len() as u16).to_le_bytes());
            expected_writer.extend_from_slice(test_string.as_bytes());

            assert_eq!(writer, expected_writer);

            let mut reader = Cursor::new(writer);
            let deserialized = XbfType::deserialize_base_type(
                &XbfMetadata::Primitive(XbfPrimitiveMetadata::String),
                &mut reader,
            )
            .unwrap();

            assert_eq!(
                deserialized,
                XbfType::Primitive(XbfPrimitive::String(test_string))
            );
        }

        #[test]
        fn bytes_works() {
            let test_bytes = vec![1, 2, 3, 4];
            let primitive = XbfPrimitive::Bytes(test_bytes.clone());
            let mut writer = vec![];

            primitive.serialize_primitive_type(&mut writer).unwrap();

            let mut expected = vec![];
            expected.extend_from_slice(&(test_bytes.len() as u16).to_le_bytes());
            expected.extend_from_slice(&test_bytes);

            assert_eq!(writer, expected);

            let mut reader = Cursor::new(writer);
            let deserialized = XbfType::deserialize_base_type(
                &XbfMetadata::Primitive(XbfPrimitiveMetadata::Bytes),
                &mut reader,
            )
            .unwrap();

            assert_eq!(
                deserialized,
                XbfType::Primitive(XbfPrimitive::Bytes(test_bytes))
            )
        }
    }

    #[test]
    fn upcast_works() {
        let primitive_type = XbfPrimitive::I32(69);
        let ref_primitive_type = &primitive_type;

        assert_eq!(
            XbfType::Primitive(primitive_type.clone()),
            ref_primitive_type.to_base_type() // ref_primitive_type.to_base_type()
        );
        assert_eq!(
            XbfType::Primitive(primitive_type.clone()),
            primitive_type.into_base_type()
        );
    }

    macro_rules! from_native_test {
        ($ty:ty, $xbf_type:tt, $test_num:expr) => {
            let value: $ty = $test_num;
            let primitive: XbfPrimitive = value.clone().into();
            assert_eq!(primitive, XbfPrimitive::$xbf_type(value));
        };
    }

    #[test]
    fn from_native_works() {
        from_native_test!(bool, Bool, true);
        from_native_test!(bool, Bool, false);
        from_native_test!(u8, U8, 42);
        from_native_test!(u16, U16, 42);
        from_native_test!(u32, U32, 42);
        from_native_test!(u64, U64, 42);
        from_native_test!(u128, U128, 42);
        from_native_test!(i8, I8, 42);
        from_native_test!(i16, I16, 42);
        from_native_test!(i32, I32, 42);
        from_native_test!(i64, I64, 42);
        from_native_test!(i128, I128, 42);
        from_native_test!(f32, F32, 42.0);
        from_native_test!(f64, F64, 42.0);
        from_native_test!(Vec<u8>, Bytes, vec![1, 2, 3, 4]);
        from_native_test!(String, String, "Hello World".to_string());
    }

    macro_rules! primitive_metadata_from_primitive_test {
        ($xbf_type:tt, $test_val:expr) => {
            assert_eq!(
                XbfPrimitiveMetadata::from(&XbfPrimitive::$xbf_type($test_val)),
                XbfPrimitiveMetadata::$xbf_type
            );
            assert_eq!(
                XbfPrimitive::$xbf_type($test_val).get_metadata(),
                XbfPrimitiveMetadata::$xbf_type
            );
        };
    }

    #[test]
    fn primitve_metadata_from_primitive_works() {
        primitive_metadata_from_primitive_test!(Bool, true);
        primitive_metadata_from_primitive_test!(U8, 1);
        primitive_metadata_from_primitive_test!(U16, 1);
        primitive_metadata_from_primitive_test!(U32, 1);
        primitive_metadata_from_primitive_test!(U64, 1);
        primitive_metadata_from_primitive_test!(U128, 1);
        primitive_metadata_from_primitive_test!(U256, [1, 2, 3, 4]);
        primitive_metadata_from_primitive_test!(I8, 1);
        primitive_metadata_from_primitive_test!(I16, 1);
        primitive_metadata_from_primitive_test!(I32, 1);
        primitive_metadata_from_primitive_test!(I64, 1);
        primitive_metadata_from_primitive_test!(I128, 1);
        primitive_metadata_from_primitive_test!(I256, [1, 2, 3, 4]);
        primitive_metadata_from_primitive_test!(F32, 1.0);
        primitive_metadata_from_primitive_test!(F64, 1.0);
        primitive_metadata_from_primitive_test!(Bytes, vec![1, 2, 3, 4]);
        primitive_metadata_from_primitive_test!(String, "Hello World".to_string());
    }
}
