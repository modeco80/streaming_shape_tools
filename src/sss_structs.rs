//! Low-level SSSF format chunks.
use u24::u24;

/// An enum with a undefined value, even a #[repr(xxxinttype)]
/// value, will result in undefined behavior if a value placed
/// in it does not have an enumerator. Therefore, we're forced to
/// do this. It's stupid. But it should work.
pub mod chunk_types {
    /// A DXT1 compressed image.
    pub const DXT1: u8 = 0x60;

    /// A full name chunk. Has.. a full name.
    pub const FULL_NAME: u8 = 0x70;
}

/// The header of a SSSF chunk.
#[derive(Debug)]
#[repr(C)]
pub struct SssfChunkHeader {
    /// The chunk type. See the [chunk_types] module
    /// for allowed/known values of this member.
    pub chunk_type: u8,

    /// Offset to reach the start of the next chunk.
    pub next: [u8; 3],
}

impl SssfChunkHeader {
    /// Returns the offset to the next chunk.
    /// If this is 0, then this is the last chunk
    /// in the streaming shape frame chunk.
    pub const fn next(&self) -> u32 {
        u24::from_le_bytes(self.next).into_u32()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct SssfFrameChunkHeader {
    /// The width of this frame.
    pub width: u16,

    /// The height of this frame.
    pub height: u16,

    /// padding.
    pub padding: [u32; 2],
}

#[repr(C)]
pub struct SssfImageChunkFooter {
    pub padding: [u16; 6],
}

#[repr(C)]
pub struct SssfFullNameChunk {
    /// The name of this chunk's frame.
    pub name: [u8; 0xc],
}

/// The [squish::Format] that all SSSF files are encoded as.
pub const SSSF_FORMAT: squish::Format = squish::Format::Bc1;
