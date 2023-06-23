use crate::{
    util::{read_string, write_string},
    XbfPrimitiveMetadata, XbfTypeUpcast,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

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
    String(String),
}

impl XbfPrimitive {
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
            XbfPrimitive::String(x) => write_string(x, writer),
        }
    }

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
            XbfPrimitiveMetadata::String => read_string(reader).map(XbfPrimitive::String),
        }
    }
}

impl XbfTypeUpcast for XbfPrimitive {}

macro_rules! xbf_primitive_from_native_impl {
    ($ty:ty, $xbf_type:tt) => {
        impl From<$ty> for XbfPrimitive {
            fn from(x: $ty) -> Self {
                XbfPrimitive::$xbf_type(x)
            }
        }
    };
}

xbf_primitive_from_native_impl!(bool, Bool);
xbf_primitive_from_native_impl!(u8, U8);
xbf_primitive_from_native_impl!(u16, U16);
xbf_primitive_from_native_impl!(u32, U32);
xbf_primitive_from_native_impl!(u64, U64);
xbf_primitive_from_native_impl!(u128, U128);
xbf_primitive_from_native_impl!(i8, I8);
xbf_primitive_from_native_impl!(i16, I16);
xbf_primitive_from_native_impl!(i32, I32);
xbf_primitive_from_native_impl!(i64, I64);
xbf_primitive_from_native_impl!(i128, I128);
xbf_primitive_from_native_impl!(f32, F32);
xbf_primitive_from_native_impl!(f64, F64);
xbf_primitive_from_native_impl!(String, String);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{XbfMetadata, XbfType};
    use std::io::Cursor;

    macro_rules! serde_primitive_test {
        ($xbf_type:tt, $test_num:expr) => {
            let primitive = XbfPrimitive::$xbf_type($test_num);
            let mut writer = Vec::new();

            primitive.serialize_primitive_type(&mut writer).unwrap();

            let expected = $test_num.to_le_bytes();
            assert_eq!(writer, expected);

            let mut reader = Cursor::new(writer);

            let metadata = XbfMetadata::Primitive(XbfPrimitiveMetadata::$xbf_type);
            let expected = XbfType::Primitive(XbfPrimitive::$xbf_type($test_num));

            let primitive = XbfType::deserialize_base_type(&metadata, &mut reader).unwrap();
            assert_eq!(primitive, expected);
        };
    }

    #[test]
    fn bool_serde_works() {
        let xbf_true = XbfPrimitive::Bool(true);
        let xbf_false = XbfPrimitive::Bool(false);
        let mut writer = Vec::new();

        xbf_true.serialize_primitive_type(&mut writer).unwrap();
        xbf_false.serialize_primitive_type(&mut writer).unwrap();

        assert_eq!(writer, vec![1, 0]);

        let mut reader = Cursor::new(writer);
        let metadata = XbfMetadata::Primitive(XbfPrimitiveMetadata::Bool);

        let true_type = XbfType::deserialize_base_type(&metadata, &mut reader).unwrap();
        let false_type = XbfType::deserialize_base_type(&metadata, &mut reader).unwrap();

        assert_eq!(true_type, XbfType::Primitive(XbfPrimitive::Bool(true)));
        assert_eq!(false_type, XbfType::Primitive(XbfPrimitive::Bool(false)));
    }

    #[test]
    fn unsigned_nums_serde_works() {
        serde_primitive_test!(U8, 42u8);
        serde_primitive_test!(U16, 420u16);
        serde_primitive_test!(U32, 100_000u32);
        serde_primitive_test!(U64, 100_000_000u64);
        serde_primitive_test!(U128, 18_446_744_073_709_551_617u128);
    }

    #[test]
    fn u256_serde_works() {
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
    fn signed_nums_serde_works() {
        serde_primitive_test!(I8, 42i8);
        serde_primitive_test!(I16, 420i16);
        serde_primitive_test!(I32, 100_000i32);
        serde_primitive_test!(I64, 100_000_000i64);
        serde_primitive_test!(I128, 18_446_744_073_709_551_617i128);
    }

    #[test]
    fn i256_serde_works() {
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
    fn floating_point_serde_works() {
        serde_primitive_test!(F32, 69.0f32);
        serde_primitive_test!(F64, 69.0f64);
    }

    #[test]
    fn string_serialize_works() {
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

    macro_rules! primitive_from_native_test {
        ($ty:ty, $xbf_type:tt, $test_num:expr) => {
            let value: $ty = $test_num;
            let primitive: XbfPrimitive = value.clone().into();
            assert_eq!(primitive, XbfPrimitive::$xbf_type(value));
        };
    }

    #[test]
    fn primitive_from_native_works() {
        primitive_from_native_test!(bool, Bool, true);
        primitive_from_native_test!(bool, Bool, false);
        primitive_from_native_test!(u8, U8, 42);
        primitive_from_native_test!(u16, U16, 42);
        primitive_from_native_test!(u32, U32, 42);
        primitive_from_native_test!(u64, U64, 42);
        primitive_from_native_test!(u128, U128, 42);
        primitive_from_native_test!(i8, I8, 42);
        primitive_from_native_test!(i16, I16, 42);
        primitive_from_native_test!(i32, I32, 42);
        primitive_from_native_test!(i64, I64, 42);
        primitive_from_native_test!(i128, I128, 42);
        primitive_from_native_test!(f32, F32, 42.0);
        primitive_from_native_test!(f64, F64, 42.0);
        primitive_from_native_test!(String, String, "Hello World".to_string());
    }

    macro_rules! primitive_metadata_from_primitive_test {
        ($xbf_type:tt, $test_val:expr) => {
            assert_eq!(
                XbfPrimitiveMetadata::from(&XbfPrimitive::$xbf_type($test_val)),
                XbfPrimitiveMetadata::$xbf_type
            )
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
        primitive_metadata_from_primitive_test!(String, "Hello World".to_string());
    }
}
