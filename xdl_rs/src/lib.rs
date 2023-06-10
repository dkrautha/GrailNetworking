mod xdl_primitive;
mod xdl_struct;
mod xdl_vec;

//use byteorder::WriteBytesExt;
use std::io::{self, Read, Write};
use xdl_primitive::{XdlPrimitive, XdlPrimitiveMetadata};
use xdl_struct::{XdlStruct, XdlStructMetadata};
use xdl_vec::{XdlVec, XdlVecMetadata};

trait Serialize {
    fn serialize(&self, writer: &mut impl Write) -> io::Result<()>;
}

trait DeserializeType {
    fn deserialize(reader: &mut impl Read) -> io::Result<(XdlMetadata, XdlType)>;
    fn deserialize_with_metadata(
        metadata: &XdlMetadata,
        reader: &mut impl Read,
    ) -> io::Result<XdlType>;
}

trait DeserializeMetadata {
    fn deserialize(reader: &mut impl Read) -> io::Result<XdlMetadata>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum XdlMetadata {
    Primitive(XdlPrimitiveMetadata),
    Vec(XdlVecMetadata),
    Struct(XdlStructMetadata),
}

impl Serialize for XdlMetadata {
    fn serialize(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            XdlMetadata::Primitive(x) => x.serialize(writer),
            XdlMetadata::Vec(x) => x.serialize(writer),
            XdlMetadata::Struct(_x) => todo!(),
        }
    }
}

impl From<&XdlType> for XdlMetadata {
    fn from(value: &XdlType) -> Self {
        match value {
            XdlType::Primitive(x) => XdlMetadata::Primitive(x.into()),
            XdlType::Vec(_) => todo!(),
            XdlType::Struct(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum XdlType {
    Primitive(XdlPrimitive),
    Vec(XdlVec),
    Struct(XdlStruct),
}
