use crate::{
    xbf_primitive::XbfPrimitiveMetadata,
    xbf_struct::{XbfStructMetadata, STRUCT_METADATA_DISCRIMINANT},
    xbf_vec::{XbfVecMetadata, VEC_METADATA_DISCRIMINANT},
    XbfType,
};
use byteorder::ReadBytesExt;
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XbfMetadata {
    Primitive(XbfPrimitiveMetadata),
    Vec(XbfVecMetadata),
    Struct(XbfStructMetadata),
}

impl XbfMetadata {
    pub fn serialize_base_metadata(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            XbfMetadata::Primitive(x) => x.serialize_primitive_metadata(writer),
            XbfMetadata::Vec(x) => x.serialize_vec_metadata(writer),
            XbfMetadata::Struct(x) => x.serialize_struct_metadata(writer),
        }
    }

    pub fn deserialize_base_metadata(reader: &mut impl Read) -> io::Result<XbfMetadata> {
        let discriminant = reader.read_u8()?;
        if let Ok(x) = XbfPrimitiveMetadata::try_from(discriminant) {
            Ok(XbfMetadata::Primitive(x))
        } else if discriminant == VEC_METADATA_DISCRIMINANT {
            Ok(XbfVecMetadata::deserialize_vec_metadata(reader)?.to_base_metadata())
        } else if discriminant == STRUCT_METADATA_DISCRIMINANT {
            Ok(XbfStructMetadata::deserialize_struct_metadata(reader)?.to_base_metadata())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown metadata discriminant {}", discriminant),
            ))
        }
    }
}

impl From<XbfPrimitiveMetadata> for XbfMetadata {
    fn from(value: XbfPrimitiveMetadata) -> Self {
        XbfMetadata::Primitive(value)
    }
}

impl From<&XbfPrimitiveMetadata> for XbfMetadata {
    fn from(value: &XbfPrimitiveMetadata) -> Self {
        XbfMetadata::Primitive(*value)
    }
}

impl From<XbfVecMetadata> for XbfMetadata {
    fn from(value: XbfVecMetadata) -> Self {
        XbfMetadata::Vec(value)
    }
}

impl From<&XbfVecMetadata> for XbfMetadata {
    fn from(value: &XbfVecMetadata) -> Self {
        XbfMetadata::Vec(value.clone())
    }
}

impl From<XbfStructMetadata> for XbfMetadata {
    fn from(value: XbfStructMetadata) -> Self {
        XbfMetadata::Struct(value)
    }
}

impl From<&XbfStructMetadata> for XbfMetadata {
    fn from(value: &XbfStructMetadata) -> Self {
        XbfMetadata::Struct(value.clone())
    }
}

impl From<&XbfType> for XbfMetadata {
    fn from(value: &XbfType) -> Self {
        match value {
            XbfType::Primitive(v) => XbfPrimitiveMetadata::from(v).to_base_metadata(),
            XbfType::Vec(v) => XbfVecMetadata::from(v).to_base_metadata(),
            XbfType::Struct(v) => XbfStructMetadata::from(v).to_base_metadata(),
        }
    }
}

pub trait XbfMetadataUpcast: Into<XbfMetadata>
where
    XbfMetadata: for<'a> From<&'a Self>,
{
    fn into_base_metadata(self) -> XbfMetadata {
        self.into()
    }
    fn to_base_metadata(&self) -> XbfMetadata {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn deserialize_unknown_discriminant_works() {
        let bad_discriminant = 69u8;
        let mut reader = Cursor::new(vec![bad_discriminant]);

        let should_be_err = XbfMetadata::deserialize_base_metadata(&mut reader)
            .expect_err("should have failed deserialization");

        assert_eq!(
            should_be_err.to_string(),
            format!("Unknown metadata discriminant {bad_discriminant}")
        )
    }
}
