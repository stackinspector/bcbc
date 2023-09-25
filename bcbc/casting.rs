use super::*;

impl Type {
    pub const fn as_tag(&self) -> Tag {
        match self {
            Type::Unknown    => Tag::Unknown,
            Type::Unit       => Tag::Unit,
            Type::Bool       => Tag::Bool,
            Type::Int        => Tag::Int,
            Type::UInt       => Tag::UInt,
            Type::Float      => Tag::Float,
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
            Type::ObjectRef  => Tag::ObjectRef,
            Type::Timestamp  => Tag::Timestamp,
            Type::UInt8      => Tag::UInt8,
            Type::UInt16     => Tag::UInt16,
            Type::UInt32     => Tag::UInt32,
        }
    }
}

impl Value {
    pub const fn as_tag(&self) -> Tag {
        match self {
            Value::Unit          => Tag::Unit,
            Value::Bool(..)      => Tag::Bool,
            Value::Int(..)       => Tag::Int,
            Value::UInt(..)      => Tag::UInt,
            Value::Float(..)     => Tag::Float,
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
            Value::ObjectRef(..) => Tag::ObjectRef,
            Value::Timestamp(..) => Tag::Timestamp,
            Value::UInt8(..)     => Tag::UInt8,
            Value::UInt16(..)    => Tag::UInt16,
            Value::UInt32(..)    => Tag::UInt32,
        }
    }

    pub const fn as_htag(&self) -> HTag {
        match self {
            Value::Int(..)       => HTag::Int,
            Value::UInt(..)      => HTag::UInt,
            Value::Float(..)     => HTag::Float,
            Value::String(..)    => HTag::String,
            Value::Bytes(..)     => HTag::Bytes,
            Value::List(..)      => HTag::List,
            Value::Map(..)       => HTag::Map,
            Value::Tuple(..)     => HTag::Tuple,
            Value::CEnum(..)     => HTag::CEnum,
            Value::Enum(..)      => HTag::Enum,
            Value::Struct(..)    => HTag::Struct,

            Value::Unit          |
            Value::Bool(..)      |
            Value::Option(..)    |
            Value::Alias(..)     |
            Value::Type(..)      |
            Value::TypeId(..)    |
            Value::ObjectRef(..) |
            Value::Timestamp(..) |
            Value::UInt8(..)     |
            Value::UInt16(..)    |
            Value::UInt32(..)    => HTag::L4,
        }
    }

    pub fn as_type(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::Bool(..) => Type::Bool,
            Value::Int(..) => Type::Int,
            Value::UInt(..) => Type::UInt,
            Value::Float(..) => Type::Float,
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
            Value::ObjectRef(..) => Type::ObjectRef,
            Value::Timestamp(..) => Type::Timestamp,
            Value::UInt8(..) => Type::UInt8,
            Value::UInt16(..) => Type::UInt16,
            Value::UInt32(..) => Type::UInt32,
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

impl Value {
    pub fn from_float(v: f64) -> Value {
        Value::Float(v.to_bits())
    }
}

impl Value {
    pub fn into_unit(self) {
        if let Value::Unit = self {
            ()
        } else {
            unreachable!()
        }
    }

    pub fn into_bool(self) -> bool {
        if let Value::Bool(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_int(self) -> i64 {
        if let Value::Int(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_uint(self) -> u64 {
        if let Value::UInt(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_float(self) -> f64 {
        if let Value::Float(v) = self {
            f64::from_bits(v)
        } else {
            unreachable!()
        }
    }

    pub fn into_string(self) -> String {
        if let Value::String(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        if let Value::Bytes(v) = self {
            v
        } else {
            unreachable!()
        }
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

    pub fn into_type(self) -> Type {
        if let Value::Type(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_type_id(self) -> TypeId {
        if let Value::TypeId(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_object_ref(self) -> ObjectRef {
        if let Value::ObjectRef(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_timestamp(self) -> Timestamp {
        if let Value::Timestamp(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_uint8(self) -> u8 {
        if let Value::UInt8(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_uint16(self) -> u16 {
        if let Value::UInt16(v) = self {
            v
        } else {
            unreachable!()
        }
    }

    pub fn into_uint32(self) -> u32 {
        if let Value::UInt32(v) = self {
            v
        } else {
            unreachable!()
        }
    }
}
