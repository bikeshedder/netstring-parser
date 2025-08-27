# netstring-parser

[![Latest Version](https://img.shields.io/crates/v/netstring-parser.svg)](https://crates.io/crates/netstring-parser) [![Build Status](https://img.shields.io/github/actions/workflow/status/bikeshedder/netstring-parser/rust.yml?branch=main)](https://github.com/bikeshedder/netstring-parser/actions?query=workflow%3ARust) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.65+](https://img.shields.io/badge/rustc-1.65+-lightgray.svg "Rust 1.65+")](https://blog.rust-lang.org/2022/05/19/Rust-1.65.0/)

A **simple, zero-copy netstring parser** for Rust, designed for **incremental parsing of streaming data** with **minimal allocations**.

## Features

- ✅ Zero-copy parsing for streaming I/O
- ✅ Incremental parsing of partial data
- ✅ Safe Rust (no `unsafe` code)
- ✅ Optional non-zero-copy mode for convenience

## Zero-Copy Parsing with Streaming I/O

The parser is optimized for streaming I/O.

You can avoid unnecessary copies by using the methods `available_buffer` and `advance` to read directly into the buffer of the parser:

```rust
use std::io::{self, Read};
use netstring_parser::NetstringParser;

fn read_from_stream<R: Read>(mut reader: R) -> io::Result<()> {
    let mut parser = NetstringParser::new(128);
    loop {
        let read = reader.read(parser.available_buffer())?;
        if read == 0 {
            break;
        }
        parser.advance(read);
        while let Some(ns) = parser.parse_next().unwrap() {
            println!("Got: {}", ns);
        }
    }
    Ok(())
}
```

- `available_buffer()` gives you a mutable slice to write into.
- `advance(n)` informs the parser that n bytes were written.
- `parse_next()` retrieves the next complete netstring if available.

## Non-Zero-Copy Parsing

If zero-copy parsing is not feasible, use the `write` method to copy data into the parser’s internal buffer:

```rust
use netstring_parser::NetstringParser;

let mut parser = NetstringParser::new(64);
// Imagine this data coming in via some stream in small chunks
let chunks: &[&[u8]] = &[
    b"5:he",
    b"llo,5:wor",
    b"ld,3:bye,",
];
for chunk in chunks {
    parser.write(chunk).unwrap();
    while let Some(ns) = parser.parse_next().unwrap() {
        println!("Got: {}", ns);
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.

---

`netstring-parser` makes parsing netstrings in Rust **fast, safe, and efficient**. 🚀
