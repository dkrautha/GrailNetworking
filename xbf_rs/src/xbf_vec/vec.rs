use crate::{XbfMetadataUpcast, XbfPrimitive, XbfType, XbfTypeUpcast, XbfVecMetadata};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    default::Default,
    io::{self, Read, Write},
    ops::Index,
    rc::Rc,
    slice::{Iter, SliceIndex},
};

/// A vector type as defined by the XBF specification.
#[derive(Debug, Clone, PartialEq)]
pub struct XbfVec {
    pub(crate) metadata: XbfVecMetadata,
    elements: Rc<[XbfType]>,
}

impl XbfVec {
    /// Tries to create a new vector based on the supplied metadata.
    ///
    /// # Errors
    ///
    /// If all elements are not the same XBF type as what's specififed in the metadata,
    /// returns an [`ElementsNotHomogenousError`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into());
    /// let data = vec![XbfPrimitive::I32(42).into(), XbfPrimitive::I32(69).into()];
    /// let vec = XbfVec::new(metadata, data);
    ///
    /// // Data contains alements which are all the same type as the metdata.
    /// assert!(vec.is_ok());
    ///
    /// let metadata2 = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into());
    /// let data2 = vec![
    ///     XbfPrimitive::I32(42).into(),
    ///     XbfPrimitive::String("Hello".to_string()).into(),
    /// ];
    /// let vec2 = XbfVec::new(metadata2, data2);
    ///
    /// // Data contains alements which are not all the same type as the metdata.
    /// assert!(vec2.is_err());
    /// ```
    pub fn new(
        metadata: XbfVecMetadata,
        elements: Vec<XbfType>,
    ) -> Result<Self, ElementsNotHomogenousError> {
        let all_same_type = elements.iter().all(|x| *metadata.inner_type == x.into());
        let elements = elements.into();
        println!("{elements:?}");
        if all_same_type {
            Ok(Self { metadata, elements })
        } else {
            Err(ElementsNotHomogenousError)
        }
    }

    /// Creates a new vector with the supplied metadata and elements without checking for homogeneity.
    ///
    /// # Examples
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into());
    /// let data = vec![
    ///     XbfPrimitive::I32(42).into(),
    ///     XbfPrimitive::String("Hello".to_string()).into(),
    /// ];
    /// let vec = XbfVec::new_unchecked(metadata, data);
    ///
    /// // This is not good!
    /// assert_eq!(vec[0], XbfPrimitive::I32(42).into());
    /// assert_eq!(vec[1], XbfPrimitive::String("Hello".to_string()).into());
    /// ```
    pub fn new_unchecked(metadata: XbfVecMetadata, elements: Vec<XbfType>) -> Self {
        let elements = elements.into();
        Self { metadata, elements }
    }

    /// Serialize a vector as defined by the XBF specification.
    ///
    /// This function **does not** write out the metadata of the type. If you want to write out the
    /// metadata, convert this type to a [`XbfVecMetadata`] and call
    /// [`XbfVecMetadata::serialize_vec_metadata`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let vec = XbfVec::new(
    ///     XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into()),
    ///     vec![XbfPrimitive::I32(42).into()]
    /// ).unwrap();
    /// let mut writer = vec![];
    /// vec.serialize_vec_type(&mut writer).unwrap();
    ///
    /// let mut expected = vec![];
    /// expected.extend_from_slice(&1u16.to_le_bytes());
    /// expected.extend_from_slice(&42u32.to_le_bytes());
    /// assert_eq!(writer, expected);
    /// ```
    pub fn serialize_vec_type(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_u16::<LittleEndian>(self.elements.len() as u16)?;
        self.elements
            .iter()
            .try_for_each(|e| e.serialize_base_type(writer))
    }

    /// Deserialize a vector as defined by the XBF specification.
    ///
    /// This function **does not** read the metadata of the type from the reader. It is
    /// expected that to call this function the metadata for a type is already known, be
    /// that from reading it from the reader with
    /// [`deserialize_base_metadata`](crate::XbfMetadata::deserialize_base_metadata)
    /// or having it in some other manner.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let metadata = XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into());
    /// let mut reader = vec![];
    /// reader.extend_from_slice(&1u16.to_le_bytes());
    /// reader.extend_from_slice(&42u32.to_le_bytes());
    /// let mut reader = std::io::Cursor::new(reader);
    ///
    /// let vec = XbfVec::deserialize_vec_type(&metadata, &mut reader).unwrap();
    /// ```
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

    /// Returns the metadata of the vector.
    ///
    /// Getting the metadata returns an owned [`XbfVecMetadata`], which requires a clone to take
    /// place. This will likely be changed in the future to be more efficient.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::XbfVecMetadata;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfPrimitiveMetadata;
    ///
    /// let vec = XbfVec::new(
    ///     XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into()),
    ///     vec![XbfPrimitive::I32(42).into()]
    /// ).unwrap();
    ///
    /// let metadata = vec.get_metadata();
    ///
    /// assert_eq!(metadata, XbfVecMetadata::new(XbfPrimitiveMetadata::I32.into()));
    /// ```
    pub fn get_metadata(&self) -> XbfVecMetadata {
        self.metadata.clone()
    }

    /// Returns a reference to an element or subslice depending on the type of index.
    ///
    /// - If given a position, returns a reference to the element at that position or `None` if out
    /// of bounds.
    /// - If given a range, returns the subslice corresponding to that range, or `None` if out of
    /// bounds.
    ///
    /// # Examples
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::NativeToXbfPrimitive;
    /// use xbf_rs::XbfTypeUpcast;
    ///
    /// let v = XbfVec::from([10u32, 40, 30].as_slice());
    ///
    /// let ten = 10u32.into_xbf_primitive().into_base_type();
    /// let forty = 40u32.into_xbf_primitive().into_base_type();
    ///
    /// assert_eq!(Some(&forty), v.get(1));
    /// assert_eq!(Some(&[ten, forty][..]), v.get(0..2));
    /// assert_eq!(None, v.get(3));
    /// assert_eq!(None, v.get(0..4));
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[XbfType]>>::Output>
    where
        I: SliceIndex<[XbfType]>,
    {
        self.elements.get(index)
    }

    /// Returns an iterator over the vector.
    ///
    /// The iterator yields all items from start to end.
    ///
    /// # Examples
    /// ```rust
    /// use xbf_rs::XbfVec;
    /// use xbf_rs::XbfPrimitive;
    /// use xbf_rs::XbfType;
    ///
    /// let x = XbfVec::from([1i32, 2, 4].as_slice());
    /// let mut iterator = x.iter();
    ///
    /// fn extract_i32<'a>(x: Option<&'a XbfType>) -> Option<&'a i32> {
    ///     match x {
    ///         Some(XbfType::Primitive(XbfPrimitive::I32(n))) => Some(n),
    ///         _ => None
    ///     }
    /// }
    ///
    /// assert_eq!(extract_i32(iterator.next()), Some(&1));
    /// assert_eq!(extract_i32(iterator.next()), Some(&2));
    /// assert_eq!(extract_i32(iterator.next()), Some(&4));
    /// assert_eq!(extract_i32(iterator.next()), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &XbfType> {
        self.elements.iter()
    }
}

impl AsRef<[XbfType]> for XbfVec {
    fn as_ref(&self) -> &[XbfType] {
        self.elements.as_ref()
    }
}

impl<I> Index<I> for XbfVec
where
    I: SliceIndex<[XbfType]>,
{
    type Output = <I as SliceIndex<[XbfType]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.elements[index]
    }
}

impl<'a> IntoIterator for &'a XbfVec {
    type Item = &'a XbfType;

    type IntoIter = Iter<'a, XbfType>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<T> From<&[T]> for XbfVec
where
    for<'a> XbfPrimitive: From<&'a T>,
    XbfPrimitive: From<T>,
    T: Default,
{
    fn from(value: &[T]) -> Self {
        let primitive_metadata = XbfPrimitive::from(T::default()).get_metadata();
        let metadata = XbfVecMetadata::new(primitive_metadata.into_base_metadata());
        let elements = value
            .iter()
            .map(|x| XbfPrimitive::from(x).into_base_type())
            .collect();
        XbfVec::new_unchecked(metadata, elements)
    }
}

impl<T> FromIterator<T> for XbfVec
where
    for<'a> XbfPrimitive: From<&'a T>,
    XbfPrimitive: From<T>,
    T: Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let primitive_metadata = XbfPrimitive::from(T::default()).get_metadata();
        let metadata = XbfVecMetadata::new(primitive_metadata.into_base_metadata());
        let elements = iter
            .into_iter()
            .map(|x| XbfPrimitive::from(x).into_base_type())
            .collect();
        XbfVec::new_unchecked(metadata, elements)
    }
}

/// Error type for [`XbfVec`]
///
/// In the future this may include information about what element wasn't
/// the same type as the metadata.
/// TODO: more error information?
#[derive(Debug, PartialEq, Eq)]
pub struct ElementsNotHomogenousError;

impl XbfTypeUpcast for XbfVec {}

#[cfg(test)]
mod test {
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

    #[test]
    fn get_metdata_works() {
        let v = XbfVec::from([1i64, 2, 4].as_slice());
        let metadata = v.get_metadata();

        assert_eq!(
            metadata,
            XbfVecMetadata::new(XbfPrimitiveMetadata::I64.into())
        );
    }

    #[test]
    fn get_works() {
        let v = XbfVec::from([1i64, 2, 4].as_slice());
        assert_eq!(v.get(0), Some(&XbfType::Primitive(XbfPrimitive::I64(1))));
        assert_eq!(v.get(69), None);
    }

    #[test]
    fn iter_method_works() {
        let x = XbfVec::from([1i32, 2, 4].as_slice());
        let mut iterator = x.iter();

        fn extract_i32<'a>(x: Option<&'a XbfType>) -> Option<&'a i32> {
            match x {
                Some(XbfType::Primitive(XbfPrimitive::I32(n))) => Some(n),
                _ => None,
            }
        }

        assert_eq!(extract_i32(iterator.next()), Some(&1));
        assert_eq!(extract_i32(iterator.next()), Some(&2));
        assert_eq!(extract_i32(iterator.next()), Some(&4));
        assert_eq!(extract_i32(iterator.next()), None);
    }

    #[test]
    fn index_works() {
        let x = XbfVec::from([1i32, 2, 4].as_slice());
        assert_eq!(x[0], XbfType::Primitive(XbfPrimitive::I32(1)));
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 3 but the index is 69")]
    fn index_out_of_bounds_panics() {
        let x = XbfVec::from([1i32, 2, 4].as_slice());
        let _ = x[69];
    }

    #[test]
    fn into_iter_ref_works() {
        let x = &XbfVec::from([1i32, 2, 4].as_slice());

        for i in x.into_iter() {
            assert!(matches!(i, XbfType::Primitive(XbfPrimitive::I32(_))));
        }

        let _usable_after_loop = x;
    }

    #[test]
    fn from_iterator_and_slice_works() {
        let a = vec![1i32, 2, 4];
        let x = XbfVec::from(a.as_slice());

        let a_iter = a.into_iter();
        let x1 = XbfVec::from_iter(a_iter.clone());
        let x2 = a_iter.collect::<XbfVec>();

        assert_eq!(x, x1);
        assert_eq!(x, x2);
    }
}
