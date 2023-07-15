use crate::{xbf_primitive::XbfPrimitiveMetadata, XbfMetadata, XbfMetadataUpcast, XbfVec};
use byteorder::WriteBytesExt;
use std::{
    io::{self, Read, Write},
    rc::Rc,
};

/// The metadata discriminant for a Vec type.
///
/// This is the same for all vectors regardless of their contents. It's value should always be
/// equal to the discriminant value of a primitive string plus one.
pub const VEC_METADATA_DISCRIMINANT: u8 = XbfPrimitiveMetadata::String as u8 + 1;

/// Metadata for a Vec type.
///
/// Internally the metadata is stored on the heap to avoid having a recursive, infinitely sized
/// type on the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XbfVecMetadata {
    pub(crate) inner_type: Rc<XbfMetadata>,
}

impl XbfVecMetadata {
    /// Creates a new Vec metadata.
    ///
    /// This will cause a heap allocation with the moved inner type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::XbfMetadata;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let inner_type = XbfPrimitiveMetadata::I32.into();
    /// let metadata = XbfVecMetadata::new(inner_type);
    /// ```
    pub fn new(inner_type: XbfMetadata) -> Self {
        Self {
            inner_type: Rc::new(inner_type),
        }
    }

    /// Serialize Vec metadata as defined by the XBF specification.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitiveMetadata;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::VEC_METADATA_DISCRIMINANT;
    ///
    /// let inner_type = XbfPrimitiveMetadata::I32.into();
    /// let metadata = XbfVecMetadata::new(inner_type);
    /// let mut writer = Vec::new();
    /// metadata.serialize_vec_metadata(&mut writer).unwrap();
    ///
    /// assert_eq!(writer, [VEC_METADATA_DISCRIMINANT, XbfPrimitiveMetadata::I32 as u8]);
    /// ```
    pub fn serialize_vec_metadata(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u8(VEC_METADATA_DISCRIMINANT)?;
        self.inner_type.serialize_base_metadata(writer)
    }

    /// Deserialize Vec metadata as defined by the XBF specification.
    ///
    /// This method assumes that you know for a fact you are about to receive Vec metadata. If you
    /// do not know what sort of metadata you are receiving, use
    /// [`deserialize_base_metadata`](crate::XbfMetadata::deserialize_base_metadata).
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfPrimitiveMetadata;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::VEC_METADATA_DISCRIMINANT;
    ///
    /// let data = vec![XbfPrimitiveMetadata::I32 as u8];
    /// let mut reader = std::io::Cursor::new(data);
    ///
    /// let metadata = XbfVecMetadata::deserialize_vec_metadata(&mut reader).unwrap();
    ///
    /// assert_eq!(metadata, XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into()));
    /// ```
    pub fn deserialize_vec_metadata(reader: &mut impl Read) -> io::Result<XbfVecMetadata> {
        let inner_type = XbfMetadata::deserialize_base_metadata(reader)?;
        Ok(XbfVecMetadata::new(inner_type))
    }
}

impl XbfMetadataUpcast for XbfVecMetadata {}

impl From<&XbfVec> for XbfVecMetadata {
    fn from(value: &XbfVec) -> Self {
        Self {
            inner_type: value.metadata.inner_type.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use io::Cursor;

    #[test]
    fn primitive_metadata_serde_works() {
        let vec_i32_metadata =
            XbfVecMetadata::new(XbfMetadata::Primitive(XbfPrimitiveMetadata::I32));
        let vec_string_metadata =
            XbfVecMetadata::new(XbfMetadata::Primitive(XbfPrimitiveMetadata::String));
        let mut writer = vec![];

        vec_i32_metadata
            .serialize_vec_metadata(&mut writer)
            .unwrap();
        vec_string_metadata
            .serialize_vec_metadata(&mut writer)
            .unwrap();

        assert_eq!(
            writer,
            vec![
                VEC_METADATA_DISCRIMINANT,
                XbfPrimitiveMetadata::I32 as u8,
                VEC_METADATA_DISCRIMINANT,
                XbfPrimitiveMetadata::String as u8
            ]
        );

        let mut reader = Cursor::new(writer);

        let vec_i32_metadata = XbfMetadata::deserialize_base_metadata(&mut reader).unwrap();
        let vec_string_metadata = XbfMetadata::deserialize_base_metadata(&mut reader).unwrap();

        assert_eq!(
            vec_i32_metadata,
            XbfMetadata::Vec(XbfVecMetadata::new(XbfMetadata::Primitive(
                XbfPrimitiveMetadata::I32
            )))
        );
        assert_eq!(
            vec_string_metadata,
            XbfMetadata::Vec(XbfVecMetadata::new(XbfMetadata::Primitive(
                XbfPrimitiveMetadata::String
            )))
        )
    }

    #[test]
    fn nested_vec_metadata_serialize_works() {
        let vec_i32_metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into());
        let vec_vec_i32_metadata = XbfVecMetadata::new(vec_i32_metadata.into());
        let mut writer = vec![];

        vec_vec_i32_metadata
            .serialize_vec_metadata(&mut writer)
            .unwrap();

        assert_eq!(
            writer,
            vec![
                VEC_METADATA_DISCRIMINANT,
                VEC_METADATA_DISCRIMINANT,
                XbfPrimitiveMetadata::I32 as u8
            ]
        );

        let mut reader = Cursor::new(writer);

        let vec_vec_i32_metadata = XbfMetadata::deserialize_base_metadata(&mut reader).unwrap();

        let expected_metadata = XbfMetadata::Vec(XbfVecMetadata::new(XbfMetadata::Vec(
            XbfVecMetadata::new(XbfMetadata::Primitive(XbfPrimitiveMetadata::I32)),
        )));

        assert_eq!(vec_vec_i32_metadata, expected_metadata);
    }

    #[test]
    fn upcast_works() {
        let primitive_metadata = XbfPrimitiveMetadata::I32;
        let vec_metadata = XbfVecMetadata::new(primitive_metadata.into());
        let vec_metadata_ref = &vec_metadata;

        assert_eq!(
            XbfMetadata::Vec(vec_metadata.clone()),
            vec_metadata_ref.to_base_metadata()
        );
        assert_eq!(
            XbfMetadata::Vec(vec_metadata.clone()),
            vec_metadata.into_base_metadata()
        );
    }
}
