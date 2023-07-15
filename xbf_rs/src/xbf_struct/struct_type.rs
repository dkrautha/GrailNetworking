use crate::{XbfMetadata, XbfStructMetadata, XbfType, XbfTypeUpcast};
use std::{
    error::Error,
    fmt::Display,
    io::{self, Read, Write},
    rc::Rc,
};

/// A struct as defined by the XBF specification.
#[derive(Debug, Clone, PartialEq)]
pub struct XbfStruct {
    pub(crate) metadata: XbfStructMetadata,
    fields: Rc<[XbfType]>,
}

impl XbfStruct {
    /// Tries to create a new [`XbfStruct`] based on the supplied metadata.
    ///
    /// # Errors
    ///
    /// If all fields are not the same XBF type as what's specififed in the metadata,
    /// returns an [`StructFieldMismatchError`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xbf_rs::XbfStruct;
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    /// use xbf_rs::XbfType;
    ///
    /// let name = "test_struct".to_string();
    /// let field1_name = "a".to_string();
    /// let field1_type = XbfMetadata::Primitive(XbfPrimitiveMetadata::I32);
    /// let field2_name = "b".to_string();
    /// let field2_type = XbfMetadata::Primitive(XbfPrimitiveMetadata::U64);
    ///
    /// let metadata = XbfStructMetadata::new(name, vec![
    ///     (field1_name, field1_type),
    ///     (field2_name, field2_type),
    /// ]);
    ///
    /// let struct_type = XbfStruct::new(metadata.clone(), vec![
    ///      XbfPrimitive::I32(42).into(),
    ///      XbfPrimitive::U64(42).into(),
    /// ]);
    ///
    /// assert!(struct_type.is_ok());
    ///
    /// let struct2 = XbfStruct::new(metadata, vec![
    ///     XbfPrimitive::I32(42).into(),
    ///     XbfPrimitive::I64(42).into(),
    ///   
    /// ]);
    ///
    /// assert!(struct2.is_err());
    ///```
    pub fn new(
        metadata: XbfStructMetadata,
        fields: Vec<XbfType>,
    ) -> Result<Self, StructFieldMismatchError> {
        for ((name, expected_field_type), val) in metadata.fields.iter().zip(fields.iter()) {
            let actual_field_type = XbfMetadata::from(val);
            if *expected_field_type != actual_field_type {
                return Err(StructFieldMismatchError::new(
                    name,
                    expected_field_type,
                    &actual_field_type,
                ));
            }
        }
        let fields = fields.into();
        Ok(Self { metadata, fields })
    }

    /// Creates a new [`XbfStruct`] with the supplied metadata and fields without checking if the
    /// given fields are the correct types.
    ///
    /// # Example
    /// ```rust
    /// use xbf_rs::XbfStruct;
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    /// use xbf_rs::XbfType;
    ///
    /// let name = "test_struct".to_string();
    /// let field1_name = "a".to_string();
    /// let field1_type = XbfMetadata::Primitive(XbfPrimitiveMetadata::I32);
    /// let field2_name = "b".to_string();
    /// let field2_type = XbfMetadata::Primitive(XbfPrimitiveMetadata::U64);
    ///
    /// let metadata = XbfStructMetadata::new(name, vec![
    ///     (field1_name, field1_type),
    ///     (field2_name, field2_type),
    /// ]);
    ///
    /// let struct_type = XbfStruct::new(metadata.clone(), vec![
    ///      XbfPrimitive::I32(42).into(),
    ///      XbfPrimitive::U64(42).into(),
    /// ]);
    /// ```
    pub fn new_unchecked(metadata: XbfStructMetadata, fields: Vec<XbfType>) -> Self {
        let fields = fields.into();
        Self { metadata, fields }
    }

    /// Serialize a struct as defined by the XBF specification.
    ///
    /// This function **does not** write out the metadata of the type. If you want to write out the
    /// metadata, get the metadata with [`Self::get_metadata`] and serialize that with
    /// [`XbfStructMetadata::serialize_struct_metadata`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xbf_rs::XbfStruct;
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let val = XbfStruct::new(
    ///     XbfStructMetadata::new(
    ///         "test_struct".to_string(),
    ///         vec![(
    ///             "a".to_string(),
    ///             XbfPrimitiveMetadata::I32.into(),
    ///         )],
    ///     ),
    ///     vec![XbfPrimitive::I32(42).into()],
    /// ).expect("a valid struct");
    /// let mut writer = vec![];
    /// val.serialize_struct_type(&mut writer).unwrap();
    ///
    /// let mut expected = 42i32.to_le_bytes();
    /// assert_eq!(writer, expected);
    /// ```
    pub fn serialize_struct_type(&self, writer: &mut impl Write) -> io::Result<()> {
        self.fields
            .iter()
            .try_for_each(|f| f.serialize_base_type(writer))
    }

    /// Deserialize a struct as defined by the XBF specification.
    ///
    /// This function **does not** read the metadata of the type from the reader. It is expected
    /// that to call this function the metadata for atype is already known, be that from reading it
    /// from the reader with [`deserialize_base_metadata`](crate::XbfMetadata::deserialize_base_metadata)
    /// or having it in some other manner.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfStruct;
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// // setting up a reader with one i32 in it
    /// let mut reader = vec![];
    /// reader.extend_from_slice(&42i32.to_le_bytes());
    /// let mut reader = std::io::Cursor::new(reader);
    ///
    /// // the metadata we've gotten from somewhere describing the type
    /// let metadata = XbfStructMetadata::new(
    ///     "test_struct".to_string(),
    ///     vec![("a".to_string(), XbfPrimitiveMetadata::I32.into())],
    /// );
    ///
    /// // deserializing the struct with the given metadata
    /// let val = XbfStruct::deserialize_struct_type(&metadata, &mut reader).unwrap();
    /// ```
    pub fn deserialize_struct_type(
        metadata: &XbfStructMetadata,
        reader: &mut impl Read,
    ) -> io::Result<XbfStruct> {
        let mut struct_fields = vec![];
        for (_, field_type) in metadata.fields.iter() {
            struct_fields.push(XbfType::deserialize_base_type(field_type, reader)?);
        }
        Ok(Self::new_unchecked(metadata.clone(), struct_fields))
    }

    /// Returns the metadata of the struct.
    ///
    /// Getting the metadata returns an owned [`XbfStructMetadata`], which requires a clone to take
    /// place. This will likely be changed in the future to be mroe efficient.
    ///
    /// # Example
    ///
    /// ```rust
    /// use xbf_rs::XbfStruct;
    /// use xbf_rs::XbfStructMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let val = XbfStruct::new(
    ///     XbfStructMetadata::new(
    ///         "test_struct".to_string(),
    ///         vec![(
    ///             "a".to_string(),
    ///             XbfPrimitiveMetadata::I32.into(),
    ///         )],
    ///     ),
    ///     vec![XbfPrimitive::I32(42).into()],
    /// );
    ///
    /// let metadata = val.expect("a valid struct").get_metadata();
    ///
    /// assert_eq!(metadata, XbfStructMetadata::new("test_struct".to_string(), vec![
    ///     ("a".to_string(), XbfPrimitiveMetadata::I32.into()),
    /// ]));
    /// ```
    pub fn get_metadata(&self) -> XbfStructMetadata {
        self.metadata.clone()
    }
}

impl XbfTypeUpcast for XbfStruct {}

#[derive(Debug)]
pub struct StructFieldMismatchError(String);

impl StructFieldMismatchError {
    fn new(
        field_name: &str,
        expected_field_type: &XbfMetadata,
        actual_field_type: &XbfMetadata,
    ) -> StructFieldMismatchError {
        let s = format!("provided value for field {field_name} is of type {actual_field_type:?}, expected {expected_field_type:?}");
        StructFieldMismatchError(s)
    }
}

impl Display for StructFieldMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for StructFieldMismatchError {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{XbfMetadataUpcast, XbfPrimitive, XbfPrimitiveMetadata, XbfVec, XbfVecMetadata};
    use std::io::Cursor;

    #[test]
    fn test_struct_new_works() {
        let name = "test_struct";
        let field1_name = "a";
        let field2_name = "b";

        let metadata = XbfStructMetadata::new(
            name.to_string(),
            vec![
                (
                    field1_name.to_string(),
                    XbfPrimitiveMetadata::I32.into_base_metadata(),
                ),
                (
                    field2_name.to_string(),
                    XbfPrimitiveMetadata::U64.into_base_metadata(),
                ),
            ],
        );

        let with_correct_fields = XbfStruct::new(
            metadata,
            vec![
                XbfPrimitive::I32(42).into_base_type(),
                XbfPrimitive::U64(69).into_base_type(),
            ],
        );

        assert!(with_correct_fields.is_ok());
        with_correct_fields.expect("a valid struct");

        // TODO: accessor methods to test the contents of what's in the struct
    }

    #[test]
    fn test_struct_new_failure_works() {
        let name = "test_struct";
        let field1_name = "a";
        let field2_name = "b";

        let metadata = XbfStructMetadata::new(
            name.to_string(),
            vec![
                (
                    field1_name.to_string(),
                    XbfPrimitiveMetadata::I32.into_base_metadata(),
                ),
                (
                    field2_name.to_string(),
                    XbfPrimitiveMetadata::U64.into_base_metadata(),
                ),
            ],
        );

        let with_wrong_field1_type = XbfStruct::new(
            metadata,
            vec![
                XbfPrimitive::String("hi".to_string()).into_base_type(),
                XbfPrimitive::U64(69).into_base_type(),
            ],
        );

        assert!(with_wrong_field1_type.is_err());
        assert_eq!(
            with_wrong_field1_type.unwrap_err().to_string(),
            StructFieldMismatchError::new(
                field1_name,
                &XbfPrimitiveMetadata::I32.into_base_metadata(),
                &XbfPrimitiveMetadata::String.into_base_metadata()
            )
            .to_string()
        );
    }

    #[test]
    fn test_struct_new_unchecked_works() {
        let name = "test_struct";
        let field1_name = "a";
        let field2_name = "b";

        let metadata = XbfStructMetadata::new(
            name.to_string(),
            vec![
                (
                    field1_name.to_string(),
                    XbfPrimitiveMetadata::String.into_base_metadata(),
                ),
                (
                    field2_name.to_string(),
                    XbfPrimitiveMetadata::Bytes.into_base_metadata(),
                ),
            ],
        );

        let _blatantly_wrong_fields = XbfStruct::new_unchecked(
            metadata,
            vec![
                XbfPrimitive::I32(42).into_base_type(),
                XbfPrimitive::U64(69).into_base_type(),
            ],
        );

        // TODO: accessor methods to test the contents of what's in the struct
    }

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
        let inner_struct = XbfStruct::new(inner_struct_metadata, vec![primitive.to_base_type()])
            .expect("a valid struct");
        let my_struct = XbfStruct::new(
            outer_metadata.clone(),
            vec![
                primitive.clone().into(),
                vec.clone().into(),
                inner_struct.clone().into(),
            ],
        )
        .expect("a valid struct");

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
        )
        .expect("a valid struct");
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
