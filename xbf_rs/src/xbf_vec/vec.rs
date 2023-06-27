use crate::{XbfType, XbfTypeUpcast, XbfVecMetadata};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct XbfVec {
    pub(crate) metadata: XbfVecMetadata,
    elements: Vec<XbfType>,
}

impl XbfVec {
    pub fn new(
        metadata: XbfVecMetadata,
        elements: Vec<XbfType>,
    ) -> Result<Self, ElementsNotHomogenousError> {
        let all_same_type = elements.iter().all(|x| *metadata.inner_type == x.into());
        if all_same_type {
            Ok(Self { metadata, elements })
        } else {
            Err(ElementsNotHomogenousError)
        }
    }

    pub fn new_unchecked(metadata: XbfVecMetadata, elements: Vec<XbfType>) -> Self {
        Self { metadata, elements }
    }

    pub fn serialize_vec_type(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u16::<LittleEndian>(self.elements.len() as u16)?;
        self.elements
            .iter()
            .try_for_each(|e| e.serialize_base_type(writer))
    }

    pub fn deserialize_vec_type(
        metadata: &XbfVecMetadata,
        reader: &mut impl Read,
    ) -> io::Result<XbfVec> {
        let inner_type = &metadata.inner_type;
        let len = reader.read_u16::<LittleEndian>()? as usize;
        let mut elements = Vec::with_capacity(len);
        for _ in 0..len {
            elements.push(XbfType::deserialize_base_type(inner_type, reader)?);
        }
        Ok(XbfVec::new_unchecked(metadata.clone(), elements))
    }

    pub fn get_metadata(&self) -> XbfVecMetadata {
        self.metadata.clone()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElementsNotHomogenousError;

impl XbfTypeUpcast for XbfVec {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        xbf_primitive::{XbfPrimitive, XbfPrimitiveMetadata},
        XbfMetadataUpcast, XbfVecMetadata,
    };
    use std::io::Cursor;

    #[test]
    fn vec_new_fails_with_not_homogenous_data() {
        let data = vec![XbfPrimitive::I32(42).into(), XbfPrimitive::U32(69).into()];
        let metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into_base_metadata());
        let err = XbfVec::new(metadata, data).unwrap_err();
        assert_eq!(err, ElementsNotHomogenousError);
    }

    #[test]
    fn serialize_vec_primitive_works() {
        const TEST_NUM: i32 = 42;
        let metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into_base_metadata());
        let vec = XbfVec::new(
            metadata.clone(),
            vec![XbfType::Primitive(XbfPrimitive::I32(TEST_NUM))],
        )
        .unwrap();
        let mut writer = vec![];

        vec.serialize_vec_type(&mut writer).unwrap();

        let mut expected = vec![];
        expected.extend_from_slice(&1u16.to_le_bytes());
        expected.extend_from_slice(&TEST_NUM.to_le_bytes());

        assert_eq!(writer, expected);

        let mut reader = Cursor::new(writer);

        let deserialized = XbfVec::deserialize_vec_type(&metadata, &mut reader).unwrap();

        assert_eq!(vec, deserialized);
    }

    #[test]
    fn serialize_vec_of_vec_works() {
        const TEST_NUM: i32 = 42;
        let vec_of_i32_metadata =
            XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into_base_metadata());
        let vec_of_two_i32 = XbfVec::new(
            vec_of_i32_metadata.clone(),
            vec![
                XbfType::Primitive(XbfPrimitive::I32(TEST_NUM)),
                XbfType::Primitive(XbfPrimitive::I32(TEST_NUM)),
            ],
        )
        .unwrap();
        let vec_of_vec_of_i32 = XbfVec::new_unchecked(
            vec_of_i32_metadata.into(),
            vec![vec_of_two_i32.clone().into(), vec_of_two_i32.into()],
        );

        let mut writer = vec![];

        vec_of_vec_of_i32.serialize_vec_type(&mut writer).unwrap();

        let mut expected = vec![];
        expected.extend_from_slice(&2u16.to_le_bytes());
        expected.extend_from_slice(&2u16.to_le_bytes());
        expected.extend_from_slice(&TEST_NUM.to_le_bytes());
        expected.extend_from_slice(&TEST_NUM.to_le_bytes());
        expected.extend_from_slice(&2u16.to_le_bytes());
        expected.extend_from_slice(&TEST_NUM.to_le_bytes());
        expected.extend_from_slice(&TEST_NUM.to_le_bytes());

        assert_eq!(writer, expected);
    }

    #[test]
    fn deserialize_vec_primitive_works() {
        const TEST_NUM: i32 = 42;
        let mut data = vec![];
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&TEST_NUM.to_le_bytes());
        let mut reader = Cursor::new(data);

        let metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into());
        let expected = XbfVec::new(
            metadata.clone(),
            vec![XbfType::Primitive(XbfPrimitive::I32(TEST_NUM))],
        )
        .unwrap();

        let vec = XbfType::deserialize_base_type(&(metadata.into()), &mut reader).unwrap();

        assert_eq!(vec, expected.into());
    }

    #[test]
    fn deserialize_vec_of_vec_works() {
        const TEST_NUM: i32 = 42;
        let mut data = vec![];
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&TEST_NUM.to_le_bytes());
        data.extend_from_slice(&TEST_NUM.to_le_bytes());
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&TEST_NUM.to_le_bytes());
        data.extend_from_slice(&TEST_NUM.to_le_bytes());
        let mut reader = Cursor::new(data);

        let int_meta = XbfPrimitiveMetadata::I32;
        let inner_vec_metadata = XbfVecMetadata::new(int_meta.to_base_metadata());
        let outer_vec_metadata = XbfVecMetadata::new(inner_vec_metadata.to_base_metadata());

        let expected_inner_vec = XbfVec::new(
            inner_vec_metadata.clone(),
            vec![
                XbfType::Primitive(XbfPrimitive::I32(TEST_NUM)),
                XbfType::Primitive(XbfPrimitive::I32(TEST_NUM)),
            ],
        )
        .unwrap();

        let expected = XbfVec::new(
            outer_vec_metadata.clone(),
            vec![expected_inner_vec.clone().into(), expected_inner_vec.into()],
        )
        .unwrap();

        let vec =
            XbfType::deserialize_base_type(&(outer_vec_metadata.into()), &mut reader).unwrap();

        assert_eq!(vec, expected.into());
    }
}
