#[cfg(test)]
fn serde_unknown_metadata() {
    use std::io::Cursor;
    use xbf_rs::{XbfMetadata, XbfPrimitive, XbfPrimitiveMetadata, XbfType, XbfTypeUpcast};

    // Sender's side
    let native_i32 = 69i32;
    let primitive_i32: XbfPrimitive = native_i32.into();
    let metadata: XbfPrimitiveMetadata = primitive_i32.into();
    let base_i32 = primitive_i32.to_base_type();
    // TODO: add an explicit conversion method to metadata from types
    let metadata: XbfMetadata = (&base_i32).into();

    let mut writer = vec![];
    metadata.serialize_base_metadata(&writer).unwrap();
    base_i32.serialize_base_type(&mut writer).unwrap();

    // Receiver's side
    let mut reader = Cursor::new(writer);
    let deserialized_base_i32 = XbfType::deserialize_base_type(&mut reader).unwrap();
}
