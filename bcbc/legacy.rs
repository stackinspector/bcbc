use foundations::{byterepr::*, byterepr_struct};

byterepr_struct! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Timestamp {
        pub secs: i64,
        pub nanos: u32,
    }
}

byterepr_struct! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ObjectRef {
        pub ot: u16,
        pub oid: u64,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeId {
    Std(StdId),
    Hash([u8; 7]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StdId(u16);

#[inline]
pub const fn check_raw_stdid(n: u16) -> bool {
    let h8 = n >> 8;
    h8 != 0xff
}

impl TypeId {
    pub const fn from_u16(n: u16) -> TypeId {
        assert!(check_raw_stdid(n));
        TypeId::Std(StdId(n))
    }

    pub const fn from_u16_unchecked(n: u16) -> TypeId {
        TypeId::Std(StdId(n))
    }

    pub fn from_path(path: &str) -> TypeId {
        use core::hash::{Hasher, Hash};
        #[allow(deprecated)]
        let mut hasher = core::hash::SipHasher::new();
        path.as_bytes().hash(&mut hasher);
        TypeId::Hash(hasher.finish().to_bytes()[1..].try_into().unwrap())
    }

    pub const fn as_std(self) -> Option<StdId> {
        match self {
            Self::Std(stdid) => Some(stdid),
            _ => None,
        }
    }

    pub const fn as_std_inner(self) -> Option<u16> {
        match self {
            Self::Std(StdId(n)) => Some(n),
            _ => None,
        }
    }

    pub const fn as_hash(self) -> Option<[u8; 7]> {
        match self {
            Self::Hash(hash) => Some(hash),
            _ => None,
        }
    }
}

impl StdId {
    pub const fn from_u16(n: u16) -> StdId {
        assert!(check_raw_stdid(n));
        StdId(n)
    }

    #[inline]
    pub const fn from_u16_unchecked(n: u16) -> StdId {
        StdId(n)
    }

    #[inline]
    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

impl ByteRepr for TypeId {
    const SIZE: usize = 8;
    type Bytes = [u8; Self::SIZE];

    fn from_bytes(raw: [u8; Self::SIZE]) -> Self {
        if raw[0] == 0xff {
            TypeId::Hash(raw[1..7].try_into().unwrap())
        } else {
            TypeId::Std(StdId::from_u16(u16::from_be_bytes(raw[6..7].try_into().unwrap())))
        }
    }

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0; Self::SIZE];
        match self {
            TypeId::Std(stdid) => {
                buf[6..8].copy_from_slice(&stdid.to_u16().to_be_bytes());
            },
            TypeId::Hash(hash) => {
                buf[0] = 0xff;
                buf[1..8].copy_from_slice(hash);
            }
        }
        buf
    }
}
