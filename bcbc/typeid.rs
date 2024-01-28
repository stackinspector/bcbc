use foundations::byterepr::*;

pub const SCHEMA_ANONYMOUS: u8 = 0x00;
pub const SCHEMA_HASH: u8 = 0xff;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeId {
    Anonymous,
    Std(StdId),
    Hash(HashId),
}

impl TypeId {
    pub const fn as_h8(&self) -> u8 {
        match self {
            TypeId::Anonymous => SCHEMA_ANONYMOUS,
            TypeId::Std(StdId { schema, .. }) => *schema,
            TypeId::Hash(..) => SCHEMA_HASH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StdId {
    pub(crate) schema: u8,
    pub(crate) id: u16,
}

impl StdId {
    pub const fn schema(&self) -> u8 {
        self.schema
    }

    pub const fn id(&self) -> u16 {
        self.id
    }

    pub const fn from_inner(schema: u8, id: u16) -> Option<StdId> {
        match schema {
            SCHEMA_ANONYMOUS | SCHEMA_HASH => None,
            schema => Some(StdId { schema, id }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashId {
    pub(crate) hash: [u8; 7],
}

impl HashId {
    pub const fn hash(&self) -> [u8; 7] {
        self.hash
    }

    pub const fn from_hash(hash: [u8; 7]) -> HashId {
        HashId { hash }
    }

    #[allow(deprecated)]
    pub /* const */ fn from_path(path: &str) -> HashId {
        use core::hash::{Hash, Hasher, SipHasher};
        let mut hasher = SipHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        let hash = hash.to_bytes()[1..].try_into().unwrap();
        HashId { hash }
    }
}

const HASH_BEGIN: usize = 1;
const STD_ID_BEGIN: usize = 6;

impl ByteRepr for TypeId {
    const SIZE: usize = 8;
    type Bytes = [u8; Self::SIZE];

    fn from_bytes(bytes: Self::Bytes) -> Self {
        match bytes[0] {
            SCHEMA_HASH => {
                let hash = bytes[HASH_BEGIN..].try_into().unwrap();
                TypeId::Hash(HashId { hash })
            }
            SCHEMA_ANONYMOUS => TypeId::Anonymous,
            schema => {
                let id = bytes[STD_ID_BEGIN..].try_into().unwrap();
                let id = u16::from_bytes(id);
                TypeId::Std(StdId { schema, id })
            }
        }
    }

    fn to_bytes(&self) -> Self::Bytes {
        let mut bytes = [0; Self::SIZE];
        bytes[0] = self.as_h8();
        match self {
            TypeId::Std(std_id) => {
                bytes[STD_ID_BEGIN..].copy_from_slice(&std_id.id().to_bytes());
            }
            TypeId::Hash(hash_id) => {
                bytes[HASH_BEGIN..].copy_from_slice(&hash_id.hash());
            }
            TypeId::Anonymous => {}
        }
        bytes
    }
}
