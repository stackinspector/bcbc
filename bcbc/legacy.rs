use foundations::byterepr_struct;

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

