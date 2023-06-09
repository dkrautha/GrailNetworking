use crate::{
    base_metadata::XbfMetadataUpcast,
    util::{read_string, write_string},
    XbfMetadata, XbfStruct, VEC_METADATA_DISCRIMINANT,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

/// The metadata discriminant for a Struct type.
///
/// This is the same for all structs regardless of their contents. It's value should always be
/// equal to the discriminant value of the vector type plus one.
pub const STRUCT_METADATA_DISCRIMINANT: u8 = VEC_METADATA_DISCRIMINANT + 1;

/// Metadata for a Struct type.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct XbfStructMetadata {
    name: String,
    pub(super) fields: Vec<(String, XbfMetadata)>,
}

impl XbfStructMetadata {
    /// Creates a new [`XbfStructMetadata`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let metadata = XbfStructMetadata::new(
    ///     "test_struct".to_string(),
    ///     vec![
    ///         ("a".to_string(), XbfPrimitiveMetadata::I32.into()),
    ///         ("b".to_string(), XbfPrimitiveMetadata::U64.into()),
    ///     ],
    /// );
    ///
    /// // TODO: accessors for the name and fields? with a similar api to a hashmap?
    pub fn new(name: String, fields: Vec<(String, XbfMetadata)>) -> Self {
        Self { name, fields }
    }

    /// Serialize a struct as defined by the XBF specification.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfStruct;
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    /// use xbf_rs::STRUCT_METADATA_DISCRIMINANT;
    ///
    /// let struct_name = "test_struct".to_string();
    /// let field1_name = "a".to_string();
    /// let field2_name = "b".to_string();
    /// let metadata = XbfStructMetadata::new(
    ///     struct_name.clone(),
    ///     vec![
    ///         (field1_name.clone(), XbfPrimitiveMetadata::I32.into()),
    ///         (field2_name.clone(), XbfPrimitiveMetadata::U64.into()),
    ///     ],
    /// );
    /// let mut writer = vec![];
    ///
    /// metadata.serialize_struct_metadata(&mut writer).unwrap();
    ///
    /// let expected = (|| {
    ///     let mut v = vec![STRUCT_METADATA_DISCRIMINANT];
    ///     v.extend_from_slice((struct_name.len() as u16).to_le_bytes().as_slice());
    ///     v.extend_from_slice(struct_name.as_bytes());
    ///     v.extend_from_slice(2u16.to_le_bytes().as_slice());
    ///     v.extend_from_slice((field1_name.len() as u16).to_le_bytes().as_slice());
    ///     v.extend_from_slice(field1_name.as_bytes());
    ///     v.extend_from_slice((XbfPrimitiveMetadata::I32 as u8).to_le_bytes().as_slice());
    ///     v.extend_from_slice((field2_name.len() as u16).to_le_bytes().as_slice());
    ///     v.extend_from_slice(field2_name.as_bytes());
    ///     v.extend_from_slice((XbfPrimitiveMetadata::U64 as u8).to_le_bytes().as_slice());
    ///     v
    /// })();
    ///
    ///
    /// assert_eq!(writer, expected);
    /// ```
    pub fn serialize_struct_metadata(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u8(STRUCT_METADATA_DISCRIMINANT)?;
        write_string(&self.name, writer)?;
        writer.write_u16::<LittleEndian>(self.fields.len() as u16)?;
        self.fields.iter().try_for_each(|(name, type_)| {
            write_string(name, writer).and_then(|_| type_.serialize_base_metadata(writer))
        })
    }

    /// Deserialize struct metadata as defined by the XBF specification.
    ///
    ///This method assumes that you know for a fact you are about to receive Struct metadata. If you
    /// do not know what sort of metadata you are receiving, use
    /// [`deserialize_base_metadata`](crate::XbfMetadata::deserialize_base_metadata).
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfPrimitiveMetadata;
    /// use xbf_rs::STRUCT_METADATA_DISCRIMINANT;
    ///
    /// let struct_name = "test_struct".to_string();
    /// let field1_name = "a".to_string();
    /// let field2_name = "b".to_string();
    ///
    /// let reader = (|| {
    ///     let mut v = vec![];
    ///     v.extend_from_slice((struct_name.len() as u16).to_le_bytes().as_slice());
    ///     v.extend_from_slice(struct_name.as_bytes());
    ///     v.extend_from_slice(2u16.to_le_bytes().as_slice());
    ///     v.extend_from_slice((field1_name.len() as u16).to_le_bytes().as_slice());
    ///     v.extend_from_slice(field1_name.as_bytes());
    ///     v.extend_from_slice((XbfPrimitiveMetadata::I32 as u8).to_le_bytes().as_slice());
    ///     v.extend_from_slice((field2_name.len() as u16).to_le_bytes().as_slice());
    ///     v.extend_from_slice(field2_name.as_bytes());
    ///     v.extend_from_slice((XbfPrimitiveMetadata::U64 as u8).to_le_bytes().as_slice());
    ///     v
    /// })();
    /// let mut reader = std::io::Cursor::new(reader);
    ///
    /// let metadata = XbfStructMetadata::deserialize_struct_metadata(&mut reader).unwrap();
    ///
    /// assert_eq!(metadata, XbfStructMetadata::new(struct_name, vec![
    ///     (field1_name, XbfPrimitiveMetadata::I32.into()),
    ///     (field2_name, XbfPrimitiveMetadata::U64.into()),
    /// ]));
    pub fn deserialize_struct_metadata(reader: &mut impl Read) -> io::Result<XbfStructMetadata> {
        let name = read_string(reader)?;
        let len = reader.read_u16::<LittleEndian>()?;
        let mut fields = Vec::with_capacity(len as usize);
        for _ in 0..len {
            fields.push((
                read_string(reader)?,
                XbfMetadata::deserialize_base_metadata(reader)?,
            ))
        }
        Ok(XbfStructMetadata { name, fields })
    }
}

impl XbfMetadataUpcast for XbfStructMetadata {}

impl From<&XbfStruct> for XbfStructMetadata {
    fn from(value: &XbfStruct) -> Self {
        value.metadata.clone()
    }
}

#[cfg(test)]
mod test {
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
