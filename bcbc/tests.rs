use hex_literal::hex;
use crate::*;

#[test]
fn cases() {
    macro_rules! case {
        ($v:expr, $exp:expr) => {{
            println!("{:?}", &$v);
            let buf = $v.clone().encode();
            println!("len={}", $exp.len());
            println!("{}", hex::encode(&$exp));
            println!("len={}", buf.len());
            println!("{}", hex::encode(&buf));
            assert_eq!(&buf, &$exp);
            let v2 = Value::decode(&buf).unwrap();
            assert_eq!($v, v2);
        }};
    }

    case!(
        Value::Map((Type::U64, Type::List(Box::new(Type::String))), vec![
            (Value::U64(123), Value::List(Type::String, vec![
                Value::String("hello".to_owned()),
                Value::String("goodbye".to_owned()),
            ])),
            (Value::U64(999999), Value::List(Type::String, vec![
                Value::String("thanks".to_owned()),
                Value::String("how are you".to_owned()),
            ])),
        ]),
        hex!("
        b2 06 110e
        03 7b     a2 0e 85 68656c6c6f   87 676f6f64627965
        23 0f423f a2 0e 86 7468616e6b73 8b 686f772061726520796f75
        ")
    );

    case!(
        Value::Tuple(vec![
            Value::Unit,
            Value::Bool(false),
            Value::I64(-7777777),
            Value::U64(24393),
            Value::F64(50.0_f64.to_bits()),
            Value::String("Berylsoft".to_owned()),
            Value::Bytes(b"(\x00)".to_vec()),
            Value::Option(Type::String, Box::new(None)),
            Value::Option(Type::Bool, Box::new(Some(Value::Bool(true)))),
            Value::Alias(TypeId::Hash(HashId { hash: hex!("fedcba98765432") }), Box::new(Value::Bytes(b"\xff".to_vec()))),
            Value::CEnum(TypeId::Std(StdId { schema: 0x01, id: 0x5f50 }), 11),
            Value::Enum(TypeId::Std(StdId { schema: 0x01, id: 0x5f49 }), 5, Box::new(Value::I64(5))),
            Value::Enum(TypeId::Std(StdId { schema: 0xfe, id: 0x00aa }), 163, Box::new(Value::U64(12))),
            Value::Type(Type::List(Box::new(Type::List(Box::new(Type::Struct(TypeId::Anonymous)))))),
            Value::TypeId(TypeId::Hash(HashId { hash: hex!("fedcba98765432") })),
            Value::Option(Type::Tuple(vec![Type::I64, Type::Unit, Type::Unknown]), Box::new(Some(Value::Tuple(vec![Value::I64(9), Value::Unit, Value::Bool(true)])))),
        ]),
        hex!("
        cc 10
        0e
        1e
        2a 76adf1
        13 5f49
        1d 4049
        89 426572796c736f6674
        93 280029
        3e 0e
        4e 02 2e
        5e ff fedcba98765432 91 ff
        db 01 5f50
        e5 01 5f49 07 05
        ec a3 fe 00aa 03 0c
        6e 11 11 17 00
        7e ff fedcba98765432
        4e  13 03 0a 01 00  c3 07 09 0e 2e
        ")
    );
}

#[test]
fn err_cases() {
    macro_rules! err_case {
        ($exp:expr, $err:expr) => {{
            let err = Value::decode(&$exp).unwrap_err();
            assert_eq!(err, $err);
        }};
    }

    err_case!(
        hex!("7a ffffffffffffffff"),
        Error::BytevarIntSign { buf: [0xff; 8] }
    );

    err_case!(
        hex!("0e 000000"),
        Error::TooLong { rest: 3 }
    );

    err_case!(
        hex!("89 426572796c736f66"),
        Error::TooShort { rest: 8, expected: 9 }
    );

    err_case!(
        hex!("6e ff"),
        Error::Tag(0xff)
    );

    err_case!(
        hex!("82 ffff"),
        Error::Utf8(String::from_utf8(hex!("ffff").into()).unwrap_err())
    );

    err_case!(
        hex!("8c 00"),
        Error::ExtvarTooLong { l4: EXT8, exp_l4: 0u8.try_into().unwrap(), u: 0 }
    );

    err_case!(
        hex!("21 000001"),
        Error::BytevarLongerThanType { len: 3, nlen: 2, buf: hex!("00 00 00 00 00 00 00 01") }
    );

    err_case!(
        hex!("11 0001"),
        Error::BytevarLongerThanExpected { len: 2, nlen: 2, exp_len: 1, buf: hex!("00 00 00 00 00 00 00 01") }
    );

    err_case!(
        hex!("0a 00"),
        Error::BytevarNegZero { buf: [0; 8] }
    )
}
