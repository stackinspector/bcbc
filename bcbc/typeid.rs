pub const SCHEMA_ANONYMOUS: u8 = 0x00;
pub const SCHEMA_HASH: u8 = 0xff;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    pub const unsafe fn from_inner_unchecked(schema: u8, id: u16) -> StdId {
        StdId { schema, id }
    }

    pub const fn from_inner(schema: u8, id: u16) -> Option<StdId> {
        match schema {
            SCHEMA_ANONYMOUS | SCHEMA_HASH => None,
            schema => Some(StdId { schema, id })
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HashId {
    pub(crate) hash: u64,
}

impl HashId {
    pub const fn hash(&self) -> u64 {
        self.hash
    }

    pub const fn from_hash(hash: u64) -> HashId {
        HashId { hash }
    }

    #[allow(deprecated)]
    pub /* const */ fn from_path(path: &str) -> HashId {
        use core::hash::{Hash, Hasher, SipHasher};
        let mut hasher = SipHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        HashId { hash }
    }
}

// TODO: impl ByteRepr for sort etc.
