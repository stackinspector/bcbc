use foundations::{error_enum, num_enum};

pub type EnumVariantId = u64;
pub type TupleItemId = u8;

mod typeid;
pub use typeid::*;

pub mod legacy;
pub use legacy::*;

num_enum! {
    pub enum Tag {
        Unknown   = 0x00,
        Unit      = 0x01,
        Bool      = 0x02,
        Int       = 0x03,
        UInt      = 0x04,
        Float     = 0x05,
        String    = 0x06,
        Bytes     = 0x07,
        Option    = 0x08,
        List      = 0x09,
        Map       = 0x0a,
        Tuple     = 0x0b,
        Alias     = 0x0c,
        CEnum     = 0x0d,
        Enum      = 0x0e,
        Struct    = 0x0f,
        Type      = 0x10,
        TypeId    = 0x11,
        ObjectRef = 0x12,
        Timestamp = 0x13,
        UInt8     = 0x14,
        UInt16    = 0x15,
        UInt32    = 0x16,
    } as u8 else Error::Tag
}

num_enum! {
    pub enum HTag {
        L4        = 0x0,
        Int       = 0x1,
        UInt      = 0x2,
        Float     = 0x3,
        String    = 0x4,
        Bytes     = 0x5,
        List      = 0x6,
        Map       = 0x7,
        Tuple     = 0x8,
        CEnum     = 0x9,
        Enum      = 0xa,
        Struct    = 0xb,
    } as u8 else Error::HTag
}

num_enum! {
    pub enum LTag {
        Unit      = 0x0,
        False     = 0x1,
        True      = 0x2,
        None      = 0x3,
        Some      = 0x4,
        Alias     = 0x5,
        Type      = 0x6,
        TypeId    = 0x7,
        ObjectRef = 0x8,
        Timestamp = 0x9,
        UInt8     = 0xa,
        UInt16    = 0xb,
        UInt32    = 0xc,
    } as u8 else Error::LTag
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unknown,

    Unit,
    Bool,
    Int,
    UInt,
    Float,
    String,
    Bytes,

    Option(Box<Type>),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),

    Tuple(Vec<Type>),

    Alias(TypeId),
    CEnum(TypeId),
    Enum(TypeId),
    Struct(TypeId),

    Type,
    TypeId,
    ObjectRef,
    Timestamp,

    UInt8,
    UInt16,
    UInt32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(u64),
    String(String),
    Bytes(Vec<u8>),

    Option(Type, Box<Option<Value>>),
    List(Type, Vec<Value>),
    Map((Type, Type), Vec<(Value, Value)>),

    Tuple(Vec<Value>),

    Alias(TypeId, Box<Value>),
    CEnum(TypeId, EnumVariantId),
    Enum(TypeId, EnumVariantId, Box<Value>),
    Struct(TypeId, Vec<Value>),

    Type(Type),
    TypeId(TypeId),
    ObjectRef(ObjectRef),
    Timestamp(Timestamp),

    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
}

pub const EXT8: u8 = 0xc;
pub const EXT16: u8 = 0xd;
pub const EXT32: u8 = 0xe;
pub const EXT64: u8 = 0xf;

pub trait Schema {
    const ID: TypeId;
    fn serialize(self) -> Value;
    fn deserialize(val: Value) -> Self;
}

pub mod casting;
pub mod reader;
use reader::Error;
pub mod writer;

#[cfg(test)]
mod tests;
