use foundations::{error_enum, num_enum};

pub type EnumVariantId = u64;
pub type TupleItemId = u8;

mod typeid;
pub use typeid::*;

num_enum! {
    pub enum Tag {
        Unknown = 0x00,
        Unit    = 0x01,
        Bool    = 0x02,
        U8      = 0x03,
        U16     = 0x04,
        U32     = 0x05,
        U64     = 0x06,
        I8      = 0x07,
        I16     = 0x08,
        I32     = 0x09,
        I64     = 0x0a,
        F16     = 0x0b,
        F32     = 0x0c,
        F64     = 0x0d,
        String  = 0x0e,
        Bytes   = 0x0f,
        Option  = 0x10,
        List    = 0x11,
        Map     = 0x12,
        Tuple   = 0x13,
        Alias   = 0x14,
        CEnum   = 0x15,
        Enum    = 0x16,
        Struct  = 0x17,
        Type    = 0x18,
        TypeId  = 0x19,
    } as u8 else Error::Tag
}

num_enum! {
    pub enum H4 {
        N1     = 0x0,
        N2     = 0x1,
        N3     = 0x2,
        N4     = 0x3,
        N5     = 0x4,
        N6     = 0x5,
        N7     = 0x6,
        N8     = 0x7,
        String = 0x8,
        Bytes  = 0x9,
        List   = 0xa,
        Map    = 0xb,
        Tuple  = 0xc,
        CEnum  = 0xd,
        Enum   = 0xe,
        Struct = 0xf,
    } as u8 else Error::H4
}

impl H4 {
    pub const fn is_num(&self) -> bool {
        (*self as u8) < 0x8
    }

    pub const fn to_bytevar_u_pos(self) -> Result<usize> {
        Ok(match self {
            H4::N1 => 7,
            H4::N2 => 6,
            H4::N3 => 5,
            H4::N4 => 4,
            H4::N5 => 3,
            H4::N6 => 2,
            H4::N7 => 1,
            H4::N8 => 0,
            _ => return Err(Error::H4ToN(self)),
        })
    }

    pub const fn to_bytevar_f_pos(self) -> Result<usize> {
        Ok(match self {
            H4::N1 => 1,
            H4::N2 => 2,
            H4::N3 => 3,
            H4::N4 => 4,
            H4::N5 => 5,
            H4::N6 => 6,
            H4::N7 => 7,
            H4::N8 => 8,
            _ => return Err(Error::H4ToN(self)),
        })
    }

    pub const fn to_ext1(self) -> Result<Ext1> {
        Ok(match self {
            H4::N1 => Ext1::Unit,
            H4::N2 => Ext1::False,
            H4::N3 => Ext1::True,
            H4::N4 => Ext1::None,
            H4::N5 => Ext1::Some,
            H4::N6 => Ext1::Alias,
            H4::N7 => Ext1::Type,
            H4::N8 => Ext1::TypeId,
            _ => return Err(Error::H4ToExt1(self)),
        })
    }

    pub const fn from_bytevar_u_pos(pos: usize) -> H4 {
        match pos {
            7 => H4::N1,
            6 => H4::N2,
            5 => H4::N3,
            4 => H4::N4,
            3 => H4::N5,
            2 => H4::N6,
            1 => H4::N7,
            0 => H4::N8,
            _ => todo!(),
        }
    }

    pub const fn from_bytevar_f_pos(pos: usize) -> H4 {
        match pos {
            1 => H4::N1,
            2 => H4::N2,
            3 => H4::N3,
            4 => H4::N4,
            5 => H4::N5,
            6 => H4::N6,
            7 => H4::N7,
            8 => H4::N8,
            _ => todo!(),
        }
    }

    pub const fn from_ext1(ext1: Ext1) -> H4 {
        match ext1 {
            Ext1::Unit   => H4::N1,
            Ext1::False  => H4::N2,
            Ext1::True   => H4::N3,
            Ext1::None   => H4::N4,
            Ext1::Some   => H4::N5,
            Ext1::Alias  => H4::N6,
            Ext1::Type   => H4::N7,
            Ext1::TypeId => H4::N8,
        }
    }
}

num_enum! {
    pub enum L4 {
        U8   = 0x0,
        U16  = 0x1,
        U32  = 0x2,
        U64  = 0x3,
        I8   = 0x4,
        I16  = 0x5,
        I32  = 0x6,
        I64  = 0x7,
        F16  = 0x8,
        F32  = 0x9,
        F64  = 0xa,
        EXT1 = 0xb,
        EXT2 = 0xc,
        EXT3 = 0xd,
        EXT4 = 0xe,
        EXT5 = 0xf,
    } as u8 else Error::L4
}

#[inline]
pub const fn from_h4l4(h4: H4, l4: L4) -> u8 {
    (h4 as u8) << 4 | (l4 as u8)
}

#[inline]
pub fn to_h4l4(n: u8) -> Result<(H4, L4)> {
    Ok(((n >> 4).try_into()?, (n & 0xf).try_into()?))
}

num_enum! {
    pub enum Ext1 {
        Unit   = 0x0,
        False  = 0x1,
        True   = 0x2,
        None   = 0x3,
        Some   = 0x4,
        Alias  = 0x5,
        Type   = 0x6,
        TypeId = 0x7,
    } as u8 else Error::Ext1
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unknown,

    Unit,
    Bool,

    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    F32,
    F64,

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F16(u16),
    F32(u32),
    F64(u64),

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
}

pub const EXT8:  L4 = L4::EXT2; // 0xc
pub const EXT16: L4 = L4::EXT3; // 0xd
pub const EXT32: L4 = L4::EXT4; // 0xe
pub const EXT64: L4 = L4::EXT5; // 0xf

pub trait Schema {
    const ID: TypeId;
    fn serialize(self) -> Value;
    fn deserialize(val: Value) -> Self;
}

// TODO determine error and fatal
error_enum! {
    #[derive(Debug)]
    pub enum Error {
        FloatL4(u8),
        TooShort((usize, usize)),
        TooLong(usize),
        Tag(u8),
        H4(u8),
        L4(u8),
        Ext1(u8),
        H4ToN(H4),
        H4ToExt1(H4),
        Size(u64),
        // waiting for flow-sensitive typing implemented
        FstUnreachable,
        // TODO debug vars
        BytevarSlicing,
    } convert {
        Utf8 => std::string::FromUtf8Error,
    }
}

type Result<T> = core::result::Result<T, Error>;

pub mod casting;
pub mod reader;
pub mod writer;

#[cfg(test)]
mod tests;
