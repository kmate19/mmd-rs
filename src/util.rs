use std::io::Read;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid UTF-16LE encoding")]
    Utf16Le,
    #[error("Invalid UTF-16 encoding {0}")]
    FromUtf16(#[from] std::string::FromUtf16Error),
    #[error("Failed to decode UTF-16 character {0}")]
    DecodeUtf16(#[from] std::char::DecodeUtf16Error),
}

type Result<T> = std::result::Result<T, Error>;

/// Converts a UTF-16LE encoded byte slice to a Rust String.
///
/// This is taken from the nightly standard library with slight modifications.
///
/// Source: https://doc.rust-lang.org/stable/std/string/struct.String.html#method.from_utf16le
pub(crate) fn from_utf16le(v: &[u8]) -> Result<String> {
    // This means that the slice length must be even
    let (chunks, []) = v.as_chunks::<2>() else {
        return Err(Error::Utf16Le);
    };

    let res = match (cfg!(target_endian = "little"), unsafe {
        v.align_to::<u16>()
    }) {
        (true, ([], v, [])) => String::from_utf16(v)?,
        _ => char::decode_utf16(chunks.iter().copied().map(u16::from_le_bytes))
            .map(|r| r.map_err(Into::into))
            .collect::<Result<_>>()?,
    };

    Ok(res)
}

pub(crate) trait ReadExt: Read {
    #[inline]
    fn read_byte(&mut self) -> std::io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_i32_le(&mut self) -> std::io::Result<i32> {
        let mut buf = [0; std::mem::size_of::<i32>()];

        self.read_exact(&mut buf)?;

        Ok(i32::from_le_bytes(buf))
    }

    #[inline]
    fn read_i16_le(&mut self) -> std::io::Result<i16> {
        let mut buf = [0; std::mem::size_of::<i16>()];

        self.read_exact(&mut buf)?;

        Ok(i16::from_le_bytes(buf))
    }

    #[inline]
    fn read_f32_le(&mut self) -> std::io::Result<f32> {
        let mut buf = [0; std::mem::size_of::<f32>()];

        self.read_exact(&mut buf)?;

        Ok(f32::from_le_bytes(buf))
    }
}

impl<T: Read> ReadExt for T {}

// Helper macro to read an array of f32s from little-endian bytes
// this cannot be a function because the size needs to be a const generic parameter
// and const generics do not support expressions yet
macro_rules! f32_array_from_le_bytes {
    ($size:expr,$reader:ident) => {{
        const SIZE_FLOATS: usize = std::mem::size_of::<f32>();
        let mut bytes = [0; $size * SIZE_FLOATS];

        $reader.read_exact(&mut bytes)?;

        let chunks = bytes.as_chunks::<SIZE_FLOATS>().0;

        let floats: [f32; $size] = std::array::from_fn(|i| f32::from_le_bytes(chunks[i]));

        floats
    }};
}

pub(crate) use f32_array_from_le_bytes;
