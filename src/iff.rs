//! A Offshoot of [https://docs.rs/crate/iffc]. Partially rewritten
//! to not be a broken down overly-heap-allocating mess
//! (since seemingly the author did not understand newtypes),
//! and to not needlessly abuse operator overloading.
//!
//! Before any people who know I write C++ start, no, I
//! don't like operator overloading abuse there either, and
//! just because I write C++ doesn't mean I magically approve of it.
use std::{
    io::{IoSlice, Read, Write},
    mem::{self, MaybeUninit},
    slice,
};

/// An IFF chunk.
#[derive(Debug, Eq, PartialEq)]
pub struct Chunk {
    pub fcc: u32,
    pub data: Box<[u8]>,
}

/// An IFF chunk header.
#[repr(C)]
struct ChunkHeader {
    ckid: u32,
    size: u32,
}

/// The size of the IFF chunk header.
const CHUNK_HEADER_SIZE: usize = std::mem::size_of::<ChunkHeader>();

/// A newtype which wraps a [Read] and yields IFF chunks until it's
/// not posible to parse chunks from the Read instance anymore.
pub struct ChunkParser<T: Read>(T);

impl<T: Read> ChunkParser<T> {
    /// Create a new [ChunkParser] wrapping a given
    /// [Read] instance.
    pub fn new(r: T) -> Self {
        Self(r)
    }
}

impl<T: Read> Iterator for ChunkParser<T> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header_bytes = MaybeUninit::<ChunkHeader>::uninit();
        // SAFETY: This must initalize the whole chunk header by always reading
        // CHUNK_HEADER_SIZE bytes, therefore this will never leave any data uninitalized.
        let header = unsafe {
            if let Err(_) = self.0.read_exact(slice::from_raw_parts_mut(
                header_bytes.as_mut_ptr() as *mut u8,
                CHUNK_HEADER_SIZE,
            )) {
                // An error occured trying to read, so just stop iteration.
                return None;
            };
            header_bytes.assume_init()
        };

        // FIXME: It might be nice policy wise to allow different modes for this.
        // However, EAC/REAL STREAM_ expects this, so it should be good enough.
        let size = {
            if header.size == 0 {
                return None;
            }
            header.size as usize - CHUNK_HEADER_SIZE
        };
        let mut data = vec![0u8; size];

        match self.0.read_exact(&mut data) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("?? {:?}", e);
                return None;
            }
        };

        Some(Chunk {
            fcc: header.ckid,
            data: data.into_boxed_slice(),
        })
    }
}

// TODO: Make this a proper newtype which also
// holds its Write by movement.

pub struct ChunkWriter(Box<dyn Write>);

impl ChunkWriter {
    pub fn new(w: Box<dyn Write>) -> Self {
        Self(w)
    }

    /// Appends a chunk.
    pub fn append_chunk(&mut self, chunk: Chunk) -> std::io::Result<()> {
        let chunk_header = ChunkHeader {
            ckid: chunk.fcc,
            // For the IFF variant we're using (as mentioned before, EAC/REAL STREAM_ functions),
            // it assumes the chunk length includes the IFF chunk header's size inside of it.
            size: (chunk.data.len() + CHUNK_HEADER_SIZE) as u32,
        };

        // SAFETY: This doesn't allow reading beyond the chunk header.
        let chunk_header_slice = unsafe {
            IoSlice::new(slice::from_raw_parts(
                mem::transmute::<_, *const u8>(&chunk_header),
                CHUNK_HEADER_SIZE,
            ))
        };

        let count = self
            .0
            .write_vectored(&[chunk_header_slice, IoSlice::new(&chunk.data)]);

        match count {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
