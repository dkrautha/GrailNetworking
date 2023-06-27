use crate::{XbfMetadata, XbfStructMetadata, XbfType, XbfTypeUpcast};

#[derive(Debug, Clone, PartialEq)]
pub struct XbfStruct {
    pub(crate) metadata: XbfStructMetadata,
    fields: Vec<XbfType>,
}

impl XbfStruct {
    pub fn new(metadata: XbfStructMetadata, fields: Vec<XbfType>) -> Self {
        metadata
            .fields
            .iter()
            .zip(fields.iter())
            .all(|((_, x), y)| *x == XbfMetadata::from(y));
        Self { metadata, fields }
    }

    pub fn serialize_struct_type(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.fields
            .iter()
            .try_for_each(|f| f.serialize_base_type(writer))
    }

    pub fn deserialize_struct_type(
        metadata: &XbfStructMetadata,
        reader: &mut impl std::io::Read,
    ) -> std::io::Result<XbfStruct> {
        let mut struct_fields = vec![];
        for (_, field_type) in metadata.fields.iter() {
            struct_fields.push(XbfType::deserialize_base_type(field_type, reader)?);
        }
        Ok(Self::new(metadata.clone(), struct_fields))
    }

    pub fn get_metadata(&self) -> XbfStructMetadata {
        self.metadata.clone()
    }
}

impl XbfTypeUpcast for XbfStruct {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{XbfMetadataUpcast, XbfPrimitive, XbfPrimitiveMetadata, XbfVec, XbfVecMetadata};
    use std::io::Cursor;

    #[test]
    fn test_struct_serde_works() {
        let primitive_metadata = XbfMetadata::Primitive(XbfPrimitiveMetadata::I32);
        let vec_metadata = XbfMetadata::Vec(XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into()));
        let inner_struct_metadata = XbfStructMetadata::new(
            "test_struct".to_string(),
            vec![(
                "a".to_string(),
                XbfMetadata::Primitive(XbfPrimitiveMetadata::I32),
            )],
        );
        let outer_metadata = XbfStructMetadata::new(
            "test".to_string(),
            vec![
                ("a".to_string(), primitive_metadata),
                ("b".to_string(), vec_metadata),
                ("c".to_string(), inner_struct_metadata.to_base_metadata()),
            ],
        );

        let primitive = XbfPrimitive::I32(42);
        let vec = XbfVec::new_unchecked(
            XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into()),
            vec![primitive.to_base_type()],
        );
        let inner_struct = XbfStruct::new(inner_struct_metadata, vec![primitive.to_base_type()]);
        let my_struct = XbfStruct::new(
            outer_metadata.clone(),
            vec![
                primitive.clone().into(),
                vec.clone().into(),
                inner_struct.clone().into(),
            ],
        );

        let mut writer = vec![];
        my_struct.serialize_struct_type(&mut writer).unwrap();

        let mut expected = vec![];
        primitive.serialize_primitive_type(&mut expected).unwrap();
        vec.serialize_vec_type(&mut expected).unwrap();
        inner_struct.serialize_struct_type(&mut expected).unwrap();

        assert_eq!(writer, expected);

        let mut reader = Cursor::new(writer);
        let deserialized =
            XbfStruct::deserialize_struct_type(&outer_metadata, &mut reader).unwrap();

        assert_eq!(my_struct, deserialized);
    }

    #[test]
    fn upcast_works() {
        let my_struct = XbfStruct::new(
            XbfStructMetadata::new(
                "my_struct".to_string(),
                vec![("field1".to_string(), XbfPrimitiveMetadata::I32.into())],
            ),
            vec![XbfPrimitive::I32(42).into()],
        );
        let struct_ref = &my_struct;

        assert_eq!(
            XbfType::Struct(my_struct.clone()),
            struct_ref.to_base_type()
        );
        assert_eq!(
            XbfType::Struct(my_struct.clone()),
            my_struct.into_base_type()
        );
    }
}
