use foundations::byterepr::*;
use super::*;

struct Reader<'a> {
    bytes: &'a [u8],
}

macro_rules! num_impl {
    ($($num:tt)*) => {$(
        fn $num(&mut self) -> Result<$num> {
            self.bytes_sized().map($num::from_bytes)
        }
    )*};
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes }
    }

    fn finish(self) -> Result<()> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(Error::TooLong { rest: self.bytes.len() })
        }
    }

    fn split_out(&mut self, sz: usize) -> Result<&'a [u8]> {
        if sz < self.bytes.len() {
            let (got, rest) = self.bytes.split_at(sz);
            self.bytes = rest;
            Ok(got)
        } else {
            Err(Error::TooShort { rest: self.bytes.len(), expected: sz })
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        if !self.bytes.is_empty() /* 1 < self.bytes.len() */ {
            let (got, rest) = self.bytes.split_at(1);
            let byte = got[0];
            self.bytes = rest;
            Ok(byte)
        } else {
            Err(Error::TooShort { rest: 0, expected: 1 })
        }
    }

    #[inline]
    fn split_out_array<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        Ok(self.split_out(N)?.try_into().unwrap())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let src = self.split_out(buf.len())?;
        if buf.len() == 1 {
            buf[0] = src[0];
        } else {
            buf.copy_from_slice(src);
        }
        Ok(())
    }

    // TODO zero copy

    #[inline]
    fn bytes(&mut self, sz: usize) -> Result<Vec<u8>> {
        Ok(self.split_out(sz)?.to_vec())
    }

    #[inline]
    fn bytes_sized<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(*self.split_out_array()?)
    }

    #[inline(always)]
    fn u8(&mut self) -> Result<u8> {
        self.read_byte()
    }

    num_impl! {
        u16 u32 u64
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
            // TODO macro
            Tag::Unknown => Type::Unknown,
            Tag::Unit => Type::Unit,
            Tag::Bool => Type::Bool,
            Tag::U8 => Type::U8,
            Tag::U16 => Type::U16,
            Tag::U32 => Type::U32,
            Tag::U64 => Type::U64,
            Tag::I8 => Type::I8,
            Tag::I16 => Type::I16,
            Tag::I32 => Type::I32,
            Tag::I64 => Type::I64,
            Tag::F16 => Type::F16,
            Tag::F32 => Type::F32,
            Tag::F64 => Type::F64,
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

    fn extvar(&mut self, l4: L4) -> Result<u64> {
        let u = match l4 {
            EXT8 => self.u8()? as u64,
            EXT16 => self.u16()? as u64,
            EXT32 => self.u32()? as u64,
            EXT64 => self.u64()?,
            s => (s as u8) as u64,
        };
        let exp_l4 = if u < (EXT8 as u64) {
            (u as u8).try_into().unwrap()
        } else if u <= (u8::MAX as u64) {
            EXT8
        } else if u <= (u16::MAX as u64) {
            EXT16
        } else if u <= (u32::MAX as u64) {
            EXT32
        } else {
            EXT64
        };
        if exp_l4 != l4 {
            Err(Error::ExtvarTooLong { l4, exp_l4, u })
        } else {
            Ok(u)
        }
    }

    fn extszvar(&mut self, l4: L4) -> Result<usize> {
        let sz = self.extvar(l4)?;
        let sz = sz.try_into().map_err(|_| Fatal::ToSize(sz))?;
        if sz <= SIZE_MAX {
            Ok(sz)
        } else {
            Err(Error::TooLongLen(sz))
        }
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
        let (h4, l4) = casting::to_h4l4(self.u8()?)?;
        Ok(match h4 {
            H4::String => {
                let len = self.extszvar(l4)?;
                let b = self.bytes(len)?;
                Value::String(String::from_utf8(b)?)
            },
            H4::Bytes => {
                let len = self.extszvar(l4)?;
                let b = self.bytes(len)?;
                Value::Bytes(b)
            },
            H4::List => {
                let len = self.extszvar(l4)?;
                let t = self.ty()?;
                let s = self.val_seq(len)?;
                Value::List(t, s)
            },
            H4::Map => {
                let len = self.extszvar(l4)?;
                let tk = self.ty()?;
                let tv = self.ty()?;
                let s = self.val_seq_map(len)?;
                Value::Map((tk, tv), s)
            },
            H4::Tuple => {
                let len = self.extszvar(l4)?;
                let s = self.val_seq(len)?;
                Value::Tuple(s)
            },
            H4::CEnum => {
                let ev = self.extvar(l4)?;
                let r = self.typeid()?;
                Value::CEnum(r, ev)
            },
            H4::Enum => {
                let ev = self.extvar(l4)?;
                let r = self.typeid()?;
                let v = self.val()?;
                Value::Enum(r, ev, Box::new(v))
            },
            H4::Struct => {
                let len = self.extszvar(l4)?;
                let r = self.typeid()?;
                let s = self.val_seq(len)?;
                Value::Struct(r, s)
            },
            h4 => {
                macro_rules! bytevar_impl {
                    ($nty:tt, $rangefn:expr, $lenfn:expr) => {{
                        let len = h4.to_bytevar_len()?;
                        let mut buf = [0; 8];
                        self.read_exact(&mut buf[$rangefn(len)])?;
                        const NLEN: usize = core::mem::size_of::<$nty>();
                        if len > NLEN {
                            return Err(Error::BytevarLongerThanType { len, nlen: NLEN, buf });
                        }
                        let exp_len = $lenfn(&buf);
                        if len != exp_len {
                            return Err(Error::BytevarLongerThanExpected { len, nlen: NLEN, exp_len, buf });
                        }
                        let ubuf = buf[$rangefn(NLEN)].try_into().map_err(|_| Fatal::BytevarSlicing)?;
                        let u = <$nty>::from_bytes(ubuf);
                        (u, buf)
                    }};
                    (U: $nty:tt) => {{
                        bytevar_impl!($nty, casting::bytevar_urange, casting::bytevar_ulen)
                    }};
                    (F: $nty:tt) => {{
                        bytevar_impl!($nty, casting::bytevar_frange, casting::bytevar_flen)
                    }};
                }

                macro_rules! numl4_impl {
                    // TODO(Rust): macro on match arms
                    (
                        U {$($uname:ident $uty:tt)*}
                        I8 {$($i8name:ident $i8ty:tt)*}
                        I {$($iname:ident $pname:ident $nname:ident $iuty:tt $ity:tt)*}
                        F {$($fname:ident $fty:tt)*}
                        $($tt:tt)*
                    ) => {
                        match l4 {
                            $(L4::$uname => {
                                let (u, _) = bytevar_impl!(U: $uty);
                                Value::$uname(u)
                            })*,
                            $(L4::$i8name => {
                                let (u, _) = bytevar_impl!(U: $i8ty);
                                Value::$i8name(u)
                            })*,
                            $(L4::$pname => {
                                let (u, buf) = bytevar_impl!(U: $iuty);
                                let i = u.try_into().map_err(|_| Error::BytevarIntSign { buf })?;
                                Value::$iname(i)
                            }
                            L4::$nname => {
                                let (u, buf) = bytevar_impl!(U: $iuty);
                                if u == 0 {
                                    return Err(Error::BytevarNegZero { buf });
                                }
                                let i: $ity = u.try_into().map_err(|_| Error::BytevarIntSign { buf })?;
                                let i = -i; // since from uN cannot be iN::MIN
                                Value::$iname(i)
                            })*,
                            $(L4::$fname => {
                                let (u, _) = bytevar_impl!(F: $fty);
                                Value::$fname(u)
                            })*,
                            $($tt)*
                        }
                    };
                }

                numl4_impl! {
                    U {
                        U8 u8
                        U16 u16
                        U32 u32
                        U64 u64
                    }
                    I8 {
                        I8 i8
                    }
                    I {
                        I16 P16 N16 u16 i16
                        I32 P32 N32 u32 i32
                        I64 P64 N64 u64 i64
                    }
                    F {
                        F16 u16
                        F32 u32
                        F64 u64
                    }
                    L4::EXT1 => match h4.to_ext1()? {
                        Ext1::Unit => Value::Unit,
                        Ext1::True => Value::Bool(true),
                        Ext1::False => Value::Bool(false),
                        opt @ (Ext1::None | Ext1::Some) => {
                            let t = self.ty()?;
                            let opt = match opt {
                                Ext1::None => None,
                                Ext1::Some => Some(self.val()?),
                                _ => return Err(Fatal::FstUnreachable.into()),
                            };
                            Value::Option(t, Box::new(opt))
                        },
                        Ext1::Alias => {
                            let r = self.typeid()?;
                            let v = self.val()?;
                            Value::Alias(r, Box::new(v))
                        },
                        Ext1::Type => {
                            let t = self.ty()?;
                            Value::Type(t)
                        },
                        Ext1::TypeId => {
                            let r = self.typeid()?;
                            Value::TypeId(r)
                        },
                    },
                    L4::EXT2 => return Err(Error::Ext2NotImplemented),
                }
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

    pub fn decode_first_value(buf: &[u8]) -> (Result<Value>, &[u8]) {
        let mut reader = Reader::new(buf);
        let res = reader.val();
        (res, reader.bytes)
    }
}
