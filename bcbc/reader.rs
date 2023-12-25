use foundations::byterepr::*;
use super::*;

// region: primitives that should provide the same no-panic guarantees as the crate `untrusted`

struct Input<'a> {
    bytes: &'a [u8],
}

impl<'a> Input<'a> {
    #[inline(always)]
    fn byte(&self, pos: usize) -> Option<&u8> {
        self.bytes.get(pos)
    }

    #[inline(always)]
    fn bytes(&self, range: core::ops::Range<usize>) -> Option<Self> {
        self.bytes.get(range).map(|bytes| Input { bytes })
    }

    #[inline(always)]
    const fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline(always)]
    const fn leak(self) -> &'a [u8] {
        self.bytes
    }

    #[inline]
    fn leak_as_array<const N: usize>(self) -> &'a [u8; N] {
        self.bytes.try_into().unwrap(/* ? */)
    }
}

struct Reader<'a> {
    input: Input<'a>,
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { input: Input { bytes }, pos: 0 }
    }

    #[inline(always)]
    const fn rest_len(&self) -> usize {
        self.input.len() - self.pos
    }

    fn split_out(&mut self, size: usize) -> Result<Input<'a>> {
        let new_pos = self.pos.checked_add(size)
            .ok_or(Error::TooLongReadLen(size))?;
        let ret = self.input.bytes(self.pos..new_pos)
            .ok_or(Error::TooShort { rest: self.rest_len(), expected: size })?;
        self.pos = new_pos;
        Ok(ret)
    }

    fn read_byte(&mut self) -> Result<u8> {
        match self.input.byte(self.pos) {
            Some(b) => {
                // safe from overflow; see https://docs.rs/untrusted/0.9.0/src/untrusted/input.rs.html#39-43
                self.pos += 1;
                Ok(*b)
            },
            None => Err(Error::TooShort { rest: 0, expected: 1 })
        }
    }

    fn finish(self) -> Result<()> {
        let rest = self.rest_len();
        if rest == 0 {
            Ok(())
        } else {
            Err(Error::TooLong { rest })
        }
    }

    fn into_rest(mut self) -> Input<'a> {
        self.split_out(self.rest_len()).unwrap()
    }
}

// primitive derivatives
impl<'a> Reader<'a> {
    #[inline]
    fn split_out_array<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        Ok(self.split_out(N)?.leak_as_array())
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let src = self.split_out(buf.len())?.leak()/* ? */;
        if buf.len() == 1 {
            buf[0] = src[0];
        } else {
            buf.copy_from_slice(src);
        }
        Ok(())
    }

    // TODO zero copy

    #[inline]
    fn bytes(&mut self, sz: usize) -> Result<Box<[u8]>> {
        Ok(alloc_bytes(self.split_out(sz)?.leak()))
    }

    #[inline]
    fn string(&mut self, sz: usize) -> Result<Box<str>> {
        Ok(alloc_bytes(core::str::from_utf8(self.split_out(sz)?.leak())?))
    }

    #[inline]
    fn bytes_sized<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(*self.split_out_array()?)
    }
}

// We can't avoid allocs completely because of nested values and indefinite-length sequences.
// So we should check for allocation at sequence creates to ensure no panic.
#[inline(always)]
fn alloc_seq<T, F: FnMut(()) -> Result<T>>(size: usize, f: F) -> Result<Box<[T]>> {
    core::iter::repeat(()).take(size).map(f).collect()
}

// We can't avoid allocs completely because of strings and bytes.
// So we should check for allocation at owned bytes creates to ensure no panic.
#[inline(always)]
fn alloc_bytes<'a, T: ?Sized>(r: &'a T) -> Box<T>
where
    Box<T>: From<&'a T>
{
    Box::from(r)
}

// endregion

macro_rules! num_impl {
    ($($num:tt)*) => {$(
        fn $num(&mut self) -> Result<$num> {
            self.bytes_sized().map($num::from_bytes)
        }
    )*};
}

impl<'a> Reader<'a> {
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
    
        macro_rules! ty_impl {
            (
                direct {$($direct_name:ident)*}
                typeid {$($typeid_name:ident)*}
                $($tt:tt)*
            ) => {
                match tag {
                    $(Tag::$direct_name => Type::$direct_name,)*
                    $($tt)*
                    $(Tag::$typeid_name => {
                        let r = self.typeid()?;
                        Type::$typeid_name(r)
                    },)*
                }
            };
        }

        Ok(ty_impl! {
            direct {
                Unknown
                Unit
                Bool
                U8
                U16
                U32
                U64
                I8
                I16
                I32
                I64
                F16
                F32
                F64
                String
                Bytes
                Type
                TypeId
            }
            typeid {
                Alias
                CEnum
                Enum
                Struct
            }
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
                let size = self.u8()? as usize;
                let s = alloc_seq(size, |_| self.ty())?;
                Type::Tuple(s)
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

    fn val_seq(&mut self, size: usize) -> Result<Box<[Value]>> {
        alloc_seq(size, |_| self.val())
    }

    fn val_seq_map(&mut self, size: usize) -> Result<Box<[(Value, Value)]>> {
        alloc_seq(size, |_| {
            let k = self.val()?;
            let v = self.val()?;
            Ok((k, v))
        })
    }

    fn val(&mut self) -> Result<Value> {
        let (h4, l4) = casting::to_h4l4(self.u8()?)?;
        Ok(match h4 {
            H4::String => {
                let len = self.extszvar(l4)?;
                let b = self.string(len)?;
                Value::String(b)
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
                        // TODO using fst
                        Ext1::None => {
                            let t = self.ty()?;
                            Value::Option(t, Box::new(None))
                        }
                        Ext1::Some => {
                            let t = self.ty()?;
                            Value::Option(t, Box::new(Some(self.val()?)))
                        }
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
        (res, reader.into_rest().leak())
    }
}
