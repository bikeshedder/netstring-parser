use netstring_parser::{NetstringError, NetstringParser, WriteError};

#[test]
fn parse_simple_netstring() {
    let mut parser = NetstringParser::new(32);

    parser.write(b"5:hello,").unwrap();

    {
        let ns = parser.parse_next().unwrap().unwrap();
        assert_eq!(&*ns, b"hello");
    }

    assert!(parser.parse_next().unwrap().is_none());
}

#[test]
fn parse_multiple_netstrings() {
    let mut parser = NetstringParser::new(32);

    parser.write(b"5:hello,5:world,").unwrap();

    {
        let ns = parser.parse_next().unwrap().unwrap();
        assert_eq!(&*ns, b"hello");
    }

    {
        let ns = parser.parse_next().unwrap().unwrap();
        assert_eq!(&*ns, b"world");
    }

    assert!(parser.parse_next().unwrap().is_none());
}

#[test]
fn incomplete_netstring_returns_none() {
    let mut parser = NetstringParser::new(32);

    parser.write(b"5:hel").unwrap();

    assert!(parser.parse_next().unwrap().is_none());

    // Append the rest to complete the netstring
    parser.write(b"lo,").unwrap();
    let ns = parser.parse_next().unwrap().unwrap();
    assert_eq!(&*ns, b"hello");
}

#[test]
fn invalid_length_error() {
    let mut parser = NetstringParser::new(32);

    parser.write(b"x:bad,").unwrap();

    let err = parser.parse_next().unwrap_err();
    matches!(err, NetstringError::InvalidLength);
}

#[test]
fn invalid_missing_comma_error() {
    let mut parser = NetstringParser::new(32);

    parser.write(b"5:hello").unwrap();
    parser.parse_next().unwrap();

    parser.write(b"x").unwrap();
    let err = parser.parse_next().unwrap_err();

    matches!(err, NetstringError::InvalidData);
}

#[test]
fn append_buffer_too_small() {
    let mut parser = NetstringParser::new(5);

    let result = parser.write(b"123456"); // too big
    assert!(matches!(result, Err(WriteError::BufferTooSmall)));
}

#[test]
fn reset_parser() {
    let mut parser = NetstringParser::new(32);

    parser.write(b"5:hello,").unwrap();
    {
        let ns = parser.parse_next().unwrap().unwrap();
        assert_eq!(&*ns, b"hello");
    }

    parser.clear();
    assert!(parser.parse_next().unwrap().is_none());

    parser.write(b"5:world,").unwrap();
    let ns2 = parser.parse_next().unwrap().unwrap();
    assert_eq!(&*ns2, b"world");
}

#[test]
fn parse_multiple_netstrings_in_chunks() {
    let mut parser = NetstringParser::new(32);

    let chunks: &[&[u8]] = &[
        b"5:he",      // partial "hello,"
        b"llo,5:w",   // completes "hello," and partial "world,"
        b"orld,3:by", // completes "world," and partial "bye,"
        b"e,",        // completes "bye,"
    ];

    let expected: &[&[u8]] = &[b"hello", b"world", b"bye"];
    let mut results = Vec::new();

    for chunk in chunks {
        parser.write(chunk).unwrap();

        while let Some(ns) = parser.parse_next().unwrap() {
            results.push(ns.to_vec()); // clone the slice for testing
        }
    }

    assert_eq!(results.len(), expected.len());
    for (res, &exp) in results.iter().zip(expected) {
        assert_eq!(res.as_slice(), exp);
    }

    // Buffer should now be empty
    assert!(parser.parse_next().unwrap().is_none());
    assert!(parser.is_buffer_empty());
}
