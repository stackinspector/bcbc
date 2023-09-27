use foundations::{num_compress::*, usize_casting::*, bytes_read::*};
use super::*;

error_enum! {
    #[derive(Debug)]
    pub enum Error {
        FloatL4(u8),
        TooShort((usize, usize)),
        TooLong(usize),
        Tag(u8),
        HTag(u8),
        LTag(u8),
    } convert {
        Utf8 => std::string::FromUtf8Error,
    }
}

type Result<T> = core::result::Result<T, Error>;

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes }
    }

    fn finish(self) -> Result<()> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(Error::TooLong(self.bytes.len()))
        }
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.bytes.read(buf).map_err(Error::TooShort)
    }

    #[inline]
    fn bytes(&mut self, sz: usize) -> Result<Vec<u8>> {
        self.bytes.read_to_vec(sz).map_err(Error::TooShort)
    }

    #[inline]
    fn bytes_sized<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.bytes.read_to_array().map_err(Error::TooShort)
    }

    fn u8(&mut self) -> Result<u8> {
        self.bytes.read_byte().ok_or(Error::TooShort((0, 1)))
    }

    fn u16(&mut self) -> Result<u16> {
        self.bytes_sized().map(u16::from_be_bytes)
    }

    fn u32(&mut self) -> Result<u32> {
        self.bytes_sized().map(u32::from_be_bytes)
    }

    fn u64(&mut self) -> Result<u64> {
        self.bytes_sized().map(u64::from_be_bytes)
    }

    fn typeid(&mut self) -> Result<TypeId> {
        let h8 = self.u8()?;
        Ok(match h8 {
            SCHEMA_HASH => {
                let hash = self.bytes_sized()?;
                TypeId::Hash(HashId { hash })
            },
            SCHEMA_ANONYMOUS => {
                TypeId::Anonymous
            },
            schema => {
                let id = self.u16()?;
                TypeId::Std(StdId { schema, id })
            }
        })
    }

    fn ty(&mut self) -> Result<Type> {
        let tag = self.u8()?.try_into()?;
        Ok(match tag {
            Tag::Unknown => Type::Unknown,
            Tag::Unit => Type::Unit,
            Tag::Bool => Type::Bool,
            Tag::Int => Type::Int,
            Tag::UInt => Type::UInt,
            Tag::Float => Type::Float,
            Tag::String => Type::String,
            Tag::Bytes => Type::Bytes,
            Tag::Type => Type::Type,
            Tag::TypeId => Type::TypeId,

            Tag::Option => {
                let t = self.ty()?;
                Type::Option(Box::new(t))
            },
            Tag::List => {
                let t = self.ty()?;
                Type::List(Box::new(t))
            },
            Tag::Map => {
                let tk = self.ty()?;
                let tv = self.ty()?;
                Type::Map(Box::new(tk), Box::new(tv))
            },

            Tag::Tuple => {
                let len = self.u8()? as usize;
                let mut s = Vec::with_capacity(len);
                for _ in 0..len {
                    let t = self.ty()?;
                    s.push(t)
                }
                Type::Tuple(s)
            },

            Tag::Alias => {
                let r = self.typeid()?;
                Type::Alias(r)
            },
            Tag::CEnum => {
                let r = self.typeid()?;
                Type::CEnum(r)
            },
            Tag::Enum => {
                let r = self.typeid()?;
                Type::Enum(r)
            },
            Tag::Struct => {
                let r = self.typeid()?;
                Type::Struct(r)
            },
        })
    }

    fn with_ltag(&mut self, l4: u8) -> Result<LTag> {
        Ok(l4.try_into()?)
    }

    fn with_uvar(&mut self, l4: u8) -> Result<u64> {
        Ok(match l4 {
            EXT8 => self.u8()? as u64,
            EXT16 => self.u16()? as u64,
            EXT32 => self.u32()? as u64,
            EXT64 => self.u64()?,
            s => s as u64,
        })
    }

    fn with_ivar(&mut self, l4: u8) -> Result<i64> {
        Ok(zigzag_decode(self.with_uvar(l4)?))
    }

    fn with_szvar(&mut self, l4: u8) -> Result<usize> {
        Ok(u64_usize(self.with_uvar(l4)?))
    }

    fn with_fvar(&mut self, l4: u8) -> Result<u64> {
        if l4 > 8 { return Err(Error::FloatL4(l4)); }
        let mut buf = [0; 8];
        self.read_exact(&mut buf[0..l4 as usize])?;
        Ok(u64::from_be_bytes(buf))
    }

    fn val_seq(&mut self, size: usize) -> Result<Vec<Value>> {
        let mut s = Vec::with_capacity(size);
        for _ in 0..size {
            let v = self.val()?;
            s.push(v)
        }
        Ok(s)
    }

    fn val_seq_map(&mut self, size: usize) -> Result<Vec<(Value, Value)>> {
        let mut s = Vec::with_capacity(size);
        for _ in 0..size {
            let k = self.val()?;
            let v = self.val()?;
            s.push((k, v))
        }
        Ok(s)
    }

    fn val(&mut self) -> Result<Value> {
        let (htag, l4) = to_h4l4(self.u8()?);
        Ok(match htag.try_into()? {
            HTag::L4 => {
                let ltag = self.with_ltag(l4)?;
                match ltag {
                    LTag::Unit => Value::Unit,
                    LTag::True => Value::Bool(true),
                    LTag::False => Value::Bool(false),
                    opt @ (LTag::None | LTag::Some) => {
                        let t = self.ty()?;
                        let opt = match opt {
                            LTag::None => None,
                            LTag::Some => Some(self.val()?),
                            _ => unreachable!(), // waiting for flow-sensitive typing implemented
                        };
                        Value::Option(t, Box::new(opt))
                    },
                    LTag::Alias => {
                        let r = self.typeid()?;
                        let v = self.val()?;
                        Value::Alias(r, Box::new(v))
                    },
                    LTag::Type => {
                        let t = self.ty()?;
                        Value::Type(t)
                    },
                    LTag::TypeId => {
                        let r = self.typeid()?;
                        Value::TypeId(r)
                    },
                }
            },
            HTag::Int => {
                let i = self.with_ivar(l4)?;
                Value::Int(i)
            },
            HTag::UInt => {
                let u = self.with_uvar(l4)?;
                Value::UInt(u)
            },
            HTag::Float => {
                let f = self.with_fvar(l4)?;
                Value::Float(f)
            },
            HTag::String => {
                let len = self.with_szvar(l4)?;
                let b = self.bytes(len)?;
                Value::String(String::from_utf8(b)?)
            },
            HTag::Bytes => {
                let len = self.with_szvar(l4)?;
                let b = self.bytes(len)?;
                Value::Bytes(b)
            },
            HTag::List => {
                let len = self.with_szvar(l4)?;
                let t = self.ty()?;
                let s = self.val_seq(len)?;
                Value::List(t, s)
            },
            HTag::Map => {
                let len = self.with_szvar(l4)?;
                let tk = self.ty()?;
                let tv = self.ty()?;
                let s = self.val_seq_map(len)?;
                Value::Map((tk, tv), s)
            },
            HTag::Tuple => {
                let len = self.with_szvar(l4)?;
                let s = self.val_seq(len)?;
                Value::Tuple(s)
            },
            HTag::CEnum => {
                let ev = self.with_uvar(l4)?;
                let r = self.typeid()?;
                Value::CEnum(r, ev)
            },
            HTag::Enum => {
                let ev = self.with_uvar(l4)?;
                let r = self.typeid()?;
                let v = self.val()?;
                Value::Enum(r, ev, Box::new(v))
            },
            HTag::Struct => {
                let len = self.with_szvar(l4)?;
                let r = self.typeid()?;
                let s = self.val_seq(len)?;
                Value::Struct(r, s)
            },
        })
    }
}

impl Value {
    pub fn decode(buf: &[u8]) -> Result<Value> {
        let mut reader = Reader::new(buf);
        let val = reader.val()?;
        reader.finish()?;
        Ok(val)
    }
}
