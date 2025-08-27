#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]
#![allow(clippy::uninlined_format_args)]
use std::{ops::Deref, str::Utf8Error};

use thiserror::Error;

/// A parser for **netstrings** (length-prefixed strings of the form `len:data,`).
///
/// This parser maintains an internal buffer of received bytes. You can append
/// data to the buffer, parse complete netstrings, and discard processed data.
#[derive(Debug)]
pub struct NetstringParser {
    buf: Vec<u8>,
    len: usize,
}

impl NetstringParser {
    /// Creates a new parser with a buffer of the given size.
    pub fn new(buf_size: usize) -> Self {
        Self {
            buf: vec![0; buf_size],
            len: 0,
        }
    }

    /// Returns a mutable slice of the unused portion of the internal buffer.
    ///
    /// You can write data directly into this slice. After writing, you **must**
    /// call [`advance`] with the number of bytes actually written
    /// to update the parser's internal length.
    ///
    /// # Example
    /// ```rust,ignore
    /// let mut parser = NetstringParser::new(1024);
    /// let buf = parser.available_buffer();
    /// let bytes_written = some_io_read(buf); // hypothetical function
    /// parser.advance(bytes_written);
    /// ```
    ///
    /// [`advance`]: Self::advance
    pub fn available_buffer(&mut self) -> &mut [u8] {
        &mut self.buf[self.len..]
    }

    /// Advances the internal buffer position by `count` bytes.
    ///
    /// This method **must** be called after writing to the slice returned by
    /// [`available_buffer`] to update the parser state.
    ///
    /// [`available_buffer`]: Self::available_buffer
    pub fn advance(&mut self, count: usize) {
        self.len += count;
    }

    /// Writes data into the parser's internal buffer.
    ///
    /// # Note
    /// In most cases, you should prefer using [`available_buffer`] to get a mutable slice
    /// and [`advance`] to indicate how many bytes were written. This avoids unnecessary
    /// copying with the typical I/O methods.
    ///
    /// [`available_buffer`]: Self::available_buffer
    /// [`advance`]: Self::advance
    pub fn write(&mut self, data: &[u8]) -> Result<(), WriteError> {
        let remaining = self.buf.len() - self.len;
        if data.len() <= remaining {
            self.buf[self.len..self.len + data.len()].copy_from_slice(data);
            self.len += data.len();
            Ok(())
        } else {
            Err(WriteError::BufferTooSmall)
        }
    }

    /// Returns true if the internal buffer is full.
    pub fn is_buffer_full(&self) -> bool {
        self.len >= self.buf.len()
    }

    /// Returns true if the internal buffer is empty.
    pub fn is_buffer_empty(&self) -> bool {
        self.len == 0
    }

    /// Attempts to parse the next complete netstring from the buffer.
    ///
    /// Returns `Ok(Some(Netstring))` if a full netstring is available, `Ok(None)` if
    /// more data is needed, or an error if the data is malformed.
    pub fn parse_next<'a>(&'a mut self) -> Result<Option<Netstring<'a>>, NetstringError> {
        match parse_length(&self.buf[..self.len])? {
            None => Ok(None),
            Some((len, rest)) => {
                if rest.len() < len + 1 {
                    return Ok(None); // need more data
                }
                if rest[len] != b',' {
                    return Err(NetstringError::MissingComma);
                }
                let offset = self.len - rest.len();
                Ok(Some(Netstring {
                    parser: self,
                    offset,
                    length: len,
                }))
            }
        }
    }

    /// Clears the parser, discarding all buffered data.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Discards the first `count` bytes from the buffer.
    ///
    /// Internal helper used by [`Netstring`] when a netstring is dropped.
    fn discard(&mut self, count: usize) {
        self.buf.copy_within(count..self.len, 0);
        self.len = self.len.saturating_sub(count);
    }
}

/// This error is returned by `Netstring::parse_next`
#[derive(Debug, Error, Copy, Clone)]
pub enum NetstringError {
    /// The parsed string will be longer than the available buffer.
    #[error("String too long")]
    StringTooLong,
    /// The given data is invalid.
    #[error("Invalid data")]
    InvalidData,
    /// No colon found within the first 20 characters.
    #[error("No colon found")]
    NoColonFound,
    /// Missing comma at end of string
    #[error("Missing comma")]
    MissingComma,
    /// The length is not a decimal number
    #[error("Invalid length")]
    InvalidLength,
}

/// This error is returned by `NetstringParser::write`.
#[derive(Debug, Error, Copy, Clone)]
pub enum WriteError {
    /// Buffer is too small for the data that is to be written
    #[error("Buffer too small")]
    BufferTooSmall,
}

/// A parsed netstring slice.
///
/// Automatically discards the underlying bytes when dropped.
pub struct Netstring<'a> {
    parser: &'a mut NetstringParser,
    offset: usize,
    length: usize,
}

impl Netstring<'_> {
    /// Converts the netstring which consists of a slice of bytes
    /// to a string slice.
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self)
    }
    /// Get netstring as byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl<'a> std::fmt::Debug for Netstring<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Netstring").field(&self.as_bytes()).finish()
    }
}

impl<'a> std::fmt::Display for Netstring<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_str() {
            Ok(s) => f.write_str(s),
            Err(_) => write!(f, "<invalid utf-8: {:?}>", self.as_bytes()),
        }
    }
}

impl<'a> Deref for Netstring<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.parser.buf[self.offset..self.offset + self.length]
    }
}

impl<'a> Drop for Netstring<'a> {
    fn drop(&mut self) {
        // Consume the netstring including the trailing comma
        self.parser.discard(self.offset + self.length + 1);
    }
}

fn parse_length(input: &[u8]) -> Result<Option<(usize, &[u8])>, NetstringError> {
    let Some(colon_pos) = input.iter().position(|&b| b == b':') else {
        if input.len() > 20 {
            // It is safe to assume that if within the first 20 characters
            // no `:` appeared that the message is invalid. This would fit
            // message lengths up to 2^64 characters which is an unrealistic
            // length for a netstring anyways.
            return Err(NetstringError::NoColonFound);
        }
        return Ok(None);
    };
    let len = &input[..colon_pos];
    let rest = &input[colon_pos + 1..];
    let Ok(len) = std::str::from_utf8(len) else {
        return Err(NetstringError::InvalidLength);
    };
    let Ok(len) = len.parse::<usize>() else {
        return Err(NetstringError::InvalidLength);
    };
    Ok(Some((len, rest)))
}
