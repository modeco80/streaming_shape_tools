//! A Offshoot of [https://docs.rs/crate/iffc]. Partially rewritten
//! to not be a broken down overly-heap-allocating mess 
//! (since seemingly the author did not understand newtypes), 
//! and to not needlessly abuse operator overloading. 
//! 
//! Before any people who know I write C++ start, no, I 
//! don't like operator overloading abuse there either, and
//! just because I write C++ doesn't mean I magically approve of it.
use std::io::{IoSlice, IoSliceMut, Read, Write};

/// An IFF chunk represents a single segment of a complete IFF
/// file. Note: Even though this structure is capable of stroing
/// data upto `usize` but IFF limits that to `u32` only.
///
/// `0` — four-byte identity of chunk.
/// `1` — byte-data encapsulated inside it.
#[derive(Debug, Eq, PartialEq)]
pub struct Chunk {
    pub fcc: [u8; 4],
    pub data: Box<[u8]>,
}

/// The size of the IFF chunk header.
const CHUNK_HEADER_SIZE: usize = 8;

/// A structure which wraps a reader and parses IFF chunks and
/// behaves like an iterator which yields `IFFChunk` until
/// an entire-chunk can't be constructed.
pub struct ChunkParser<T: Read>(T);
impl<T: Read> ChunkParser<T> {
    pub fn new(r: T) -> Self {
        Self(r)
    }
}

impl<T: Read> Iterator for ChunkParser<T> {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let mut id = [0u8; 4];
        let mut size = [0u8; 4];

        if let Err(_) = self
            .0
            .read_vectored(&mut [IoSliceMut::new(&mut id), IoSliceMut::new(&mut size)])
        {
            return None;
        };

        if u32::from_le_bytes(size) as usize == 0 {
            return None;
        }

        // FIXME: It might be nice policy wise to allow different modes for this.
        // However, EAC/REAL STREAM_ expects this, so it should be good enough.
        let size = u32::from_le_bytes(size) as usize - CHUNK_HEADER_SIZE;
        //eprintln!("size {:08x}", size);
        let mut data = vec![0u8; size];

        match self.0.read_exact(&mut data) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("?? {:?}", e);
                return None;
            }
        };

        Some(Chunk {
            fcc: id,
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
        let count = self.0.write_vectored(&[
            IoSlice::new(&chunk.fcc),
            IoSlice::new(&(chunk.data.len() as u32).to_le_bytes()[..]),
            IoSlice::new(&chunk.data),
        ]);

        match count {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
