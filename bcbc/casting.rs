use super::*;

#[inline(always)]
pub const fn bytevar_urange(len: usize) -> core::ops::RangeFrom<usize> {
    (8 - len)..
}

#[inline(always)]
pub const fn bytevar_frange(len: usize) -> core::ops::RangeTo<usize> {
    ..len
}

pub fn bytevar_ulen(buf: &[u8; 8]) -> usize {
    for (i, b) in buf.iter().enumerate() {
        if *b != 0 {
            return 8 - i;
        }
    }
    1
}

pub fn bytevar_flen(buf: &[u8; 8]) -> usize {
    for (i, b) in buf.iter().rev().enumerate() {
        if *b != 0 {
            return 8 - i;
        }
    }
    1
}

#[inline]
pub const fn from_h4l4(h4: H4, l4: L4) -> u8 {
    (h4 as u8) << 4 | (l4 as u8)
}

#[inline]
pub fn to_h4l4(n: u8) -> Result<(H4, L4)> {
    Ok(((n >> 4).try_into()?, (n & 0xf).try_into()?))
}

impl H4 {
    pub const fn is_num(&self) -> bool {
        (*self as u8) < 0x8
    }

    pub const fn to_bytevar_len(self) -> FatalResult<usize> {
        Ok(match self {
            H4::N1 => 1,
            H4::N2 => 2,
            H4::N3 => 3,
            H4::N4 => 4,
            H4::N5 => 5,
            H4::N6 => 6,
            H4::N7 => 7,
            H4::N8 => 8,
            _ => return Err(Fatal::H4ToN(self)),
        })
    }

    pub const fn to_ext1(self) -> FatalResult<Ext1> {
        Ok(match self {
            H4::N1 => Ext1::Unit,
            H4::N2 => Ext1::False,
            H4::N3 => Ext1::True,
            H4::N4 => Ext1::None,
            H4::N5 => Ext1::Some,
            H4::N6 => Ext1::Alias,
            H4::N7 => Ext1::Type,
            H4::N8 => Ext1::TypeId,
            _ => return Err(Fatal::H4ToExt1(self)),
        })
    }

    pub const fn from_bytevar_len(pos: usize) -> FatalResult<H4> {
        Ok(match pos {
            1 => H4::N1,
            2 => H4::N2,
            3 => H4::N3,
            4 => H4::N4,
            5 => H4::N5,
            6 => H4::N6,
            7 => H4::N7,
            8 => H4::N8,
            _ => return Err(Fatal::NToH4(pos)),
        })
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

impl Type {
    pub const fn as_tag(&self) -> Tag {
        match self {
            Type::Unknown    => Tag::Unknown,
            Type::Unit       => Tag::Unit,
            Type::Bool       => Tag::Bool,
            Type::U8         => Tag::U8,
            Type::U16        => Tag::U16,
            Type::U32        => Tag::U32,
            Type::U64        => Tag::U64,
            Type::I8         => Tag::I8,
            Type::I16        => Tag::I16,
            Type::I32        => Tag::I32,
            Type::I64        => Tag::I64,
            Type::F16        => Tag::F16,
            Type::F32        => Tag::F32,
            Type::F64        => Tag::F64,
            Type::String     => Tag::String,
            Type::Bytes      => Tag::Bytes,
            Type::Option(..) => Tag::Option,
            Type::List(..)   => Tag::List,
            Type::Map(..)    => Tag::Map,
            Type::Tuple(..)  => Tag::Tuple,
            Type::Alias(..)  => Tag::Alias,
            Type::Enum(..)   => Tag::Enum,
            Type::CEnum(..)  => Tag::CEnum,
            Type::Struct(..) => Tag::Struct,
            Type::Type       => Tag::Type,
            Type::TypeId     => Tag::TypeId,
        }
    }
}

impl Value {
    pub const fn as_tag(&self) -> Tag {
        match self {
            Value::Unit          => Tag::Unit,
            Value::Bool(..)      => Tag::Bool,
            Value::U8(..)        => Tag::U8,
            Value::U16(..)       => Tag::U16,
            Value::U32(..)       => Tag::U32,
            Value::U64(..)       => Tag::U64,
            Value::I8(..)        => Tag::I8,
            Value::I16(..)       => Tag::I16,
            Value::I32(..)       => Tag::I32,
            Value::I64(..)       => Tag::I64,
            Value::F16(..)       => Tag::F16,
            Value::F32(..)       => Tag::F32,
            Value::F64(..)       => Tag::F64,
            Value::String(..)    => Tag::String,
            Value::Bytes(..)     => Tag::Bytes,
            Value::Option(..)    => Tag::Option,
            Value::List(..)      => Tag::List,
            Value::Map(..)       => Tag::Map,
            Value::Tuple(..)     => Tag::Tuple,
            Value::Alias(..)     => Tag::Alias,
            Value::CEnum(..)     => Tag::CEnum,
            Value::Enum(..)      => Tag::Enum,
            Value::Struct(..)    => Tag::Struct,
            Value::Type(..)      => Tag::Type,
            Value::TypeId(..)    => Tag::TypeId,
        }
    }

    pub fn as_type(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::Bool(..) => Type::Bool,
            Value::U8(..) => Type::U8,
            Value::U16(..) => Type::U16,
            Value::U32(..) => Type::U32,
            Value::U64(..) => Type::U64,
            Value::I8(..) => Type::I8,
            Value::I16(..) => Type::I16,
            Value::I32(..) => Type::I32,
            Value::I64(..) => Type::I64,
            Value::F16(..) => Type::F16,
            Value::F32(..) => Type::F32,
            Value::F64(..) => Type::F64,
            Value::String(..) => Type::String,
            Value::Bytes(..) => Type::Bytes,
            Value::Option(t, ..) => Type::Option(Box::new(t.clone())),
            Value::List(t, ..) => Type::List(Box::new(t.clone())),
            Value::Map((tk, tv), ..) => Type::Map(Box::new(tk.clone()), Box::new(tv.clone())),
            Value::Tuple(seq) => Type::Tuple(seq.iter().map(|v| v.as_type()).collect()),
            Value::Alias(r, ..) => Type::Alias(*r),
            Value::CEnum(r, ..) => Type::CEnum(*r),
            Value::Enum(r, ..) => Type::Enum(*r),
            Value::Struct(r, ..) => Type::Struct(*r),
            Value::Type(..) => Type::Type,
            Value::TypeId(..) => Type::TypeId,
        }
    }
}

impl Value {
    pub fn serialize_from<T: Schema>(val: T) -> Value {
        val.serialize()
    }

    pub fn deserialize_into<T: Schema>(self) -> T {
        T::deserialize(self)
    }
}

// impl Value {
//     pub fn from_float(v: f64) -> Value {
//         Value::Float(v.to_bits())
//     }
// }

macro_rules! into_v_impl {
    // TODO auto make fn name with concat_ident! and const case convert
    ($($fn_name:ident $variant:ident $ty:ty)*) => {$(
        pub fn $fn_name(self) -> $ty {
            if let Value::$variant(v) = self {
                v
            } else {
                unreachable!()
            }
        }
    )*};
}

impl Value {
    pub fn into_unit(self) {
        if let Value::Unit = self {
            ()
        } else {
            unreachable!()
        }
    }

    into_v_impl! {
        into_bool Bool bool
        into_u8 U8 u8
        into_u16 U16 u16
        into_u32 U32 u32
        into_u64 U64 u64
        into_i8 I8 i8
        into_i16 I16 i16
        into_i32 I32 i32
        into_i64 I64 i64
        // TODO convert?
        into_f16 F16 u16
        into_f32 F32 u32
        into_f64 F64 u64
        into_string String String
        into_bytes Bytes Vec<u8>
    }

    pub fn into_option(self) -> Option<Value> {
        if let Value::Option(_t, v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub fn into_list(self) -> Vec<Value> {
        if let Value::List(_t, s) = self {
            s
        } else {
            unreachable!()
        }
    }

    pub fn into_map(self) -> Vec<(Value, Value)> {
        if let Value::Map(_t, s) = self {
            s
        } else {
            unreachable!()
        }
    }

    pub fn into_tuple(self) -> Vec<Value> {
        if let Value::Tuple(s) = self {
            s
        } else {
            unreachable!()
        }
    }

    pub fn into_alias(self) -> Value {
        if let Value::Alias(_id, v) = self {
            *v
        } else {
            unreachable!()
        }
    }

    pub fn into_c_enum(self) -> EnumVariantId {
        if let Value::CEnum(_id, ev) = self {
            ev
        } else {
            unreachable!()
        }
    }

    pub fn into_enum(self) -> (EnumVariantId, Value) {
        if let Value::Enum(_id, ev, v) = self {
            (ev, *v)
        } else {
            unreachable!()
        }
    }

    pub fn into_struct(self) -> Vec<Value> {
        if let Value::Struct(_id, s) = self {
            s
        } else {
            unreachable!()
        }
    }

    into_v_impl! {
        into_type Type Type
        into_type_id TypeId TypeId
    }
}
