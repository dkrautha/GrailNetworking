use crate::{
    base_metadata::XbfMetadataUpcast,
    util::{read_string, write_string},
    XbfMetadata, XbfStruct, VEC_METADATA_DISCRIMINANT,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

pub const STRUCT_METADATA_DISCRIMINANT: u8 = VEC_METADATA_DISCRIMINANT + 1;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct XbfStructMetadata {
    name: String,
    pub(super) fields: Vec<(String, XbfMetadata)>,
}

impl XbfStructMetadata {
    pub fn new(name: String, fields: Vec<(String, XbfMetadata)>) -> Self {
        Self { name, fields }
    }

    pub fn serialize_struct_metadata(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u8(STRUCT_METADATA_DISCRIMINANT)?;
        write_string(&self.name, writer)?;
        writer.write_u16::<LittleEndian>(self.fields.len() as u16)?;
        self.fields.iter().try_for_each(|(name, type_)| {
            write_string(name, writer).and_then(|_| type_.serialize_base_metadata(writer))
        })
    }

    pub fn deserialize_struct_metadata(reader: &mut impl Read) -> io::Result<XbfMetadata> {
        let name = read_string(reader)?;
        let len = reader.read_u16::<LittleEndian>()?;
        let mut fields = Vec::with_capacity(len as usize);
        for _ in 0..len {
            fields.push((
                read_string(reader)?,
                XbfMetadata::deserialize_base_metadata(reader)?,
            ))
        }
        Ok(XbfMetadata::Struct(XbfStructMetadata { name, fields }))
    }
}

impl XbfMetadataUpcast for XbfStructMetadata {}

impl From<&XbfStruct> for XbfStructMetadata {
    fn from(value: &XbfStruct) -> Self {
        value.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{xbf_primitive::XbfPrimitiveMetadata, XbfVecMetadata};
    use std::io::Cursor;

    #[test]
    fn metadata_serde_works() {
        let metadata = XbfStructMetadata::new(
            "test".to_string(),
            vec![
                (
                    "a".to_string(),
                    XbfMetadata::Primitive(XbfPrimitiveMetadata::I32),
                ),
                (
                    "b".to_string(),
                    XbfMetadata::Vec(XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into())),
                ),
                (
                    "c".to_string(),
                    XbfMetadata::Struct(XbfStructMetadata {
                        name: "inner".to_string(),
                        fields: vec![(
                            "d".to_string(),
                            XbfMetadata::Primitive(XbfPrimitiveMetadata::I32),
                        )],
                    }),
                ),
            ],
        );

        let mut writer = Vec::new();
        metadata.serialize_struct_metadata(&mut writer).unwrap();

        let mut expected = Vec::new();
        // disciminant
        expected.write_u8(STRUCT_METADATA_DISCRIMINANT).unwrap();
        // name
        write_string(&metadata.name, &mut expected).unwrap();
        // num of fields
        expected.write_u16::<LittleEndian>(3).unwrap();
        // field a
        write_string("a", &mut expected).unwrap();
        expected.write_u8(XbfPrimitiveMetadata::I32 as u8).unwrap();
        // field b
        write_string("b", &mut expected).unwrap();
        expected.write_u8(VEC_METADATA_DISCRIMINANT).unwrap();
        expected.write_u8(XbfPrimitiveMetadata::I32 as u8).unwrap();
        // field c
        write_string("c", &mut expected).unwrap();
        // field c is a struct, so do struct stuff again
        // discriminant
        expected.write_u8(STRUCT_METADATA_DISCRIMINANT).unwrap();
        // name
        write_string("inner", &mut expected).unwrap();
        // num_of_fields
        expected.write_u16::<LittleEndian>(1).unwrap();
        // field d
        write_string("d", &mut expected).unwrap();
        expected.write_u8(XbfPrimitiveMetadata::I32 as u8).unwrap();

        assert_eq!(expected, writer);

        let mut reader = Cursor::new(writer);
        let deserialized = XbfMetadata::deserialize_base_metadata(&mut reader).unwrap();
        assert_eq!(XbfMetadata::Struct(metadata), deserialized);
    }

    #[test]
    fn upcast_works() {
        let struct_metadata = XbfStructMetadata::new(
            "test_struct".to_string(),
            vec![("field1".to_string(), XbfPrimitiveMetadata::I32.into())],
        );
        let struct_metadata_ref = &struct_metadata;

        assert_eq!(
            XbfMetadata::Struct(struct_metadata.clone()),
            struct_metadata_ref.to_base_metadata()
        );
        assert_eq!(
            XbfMetadata::Struct(struct_metadata.clone()),
            struct_metadata.into_base_metadata()
        );
    }
}
