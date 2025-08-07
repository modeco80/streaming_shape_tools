// An enum with a undefined value, even a #[repr(xxxinttype)]
// value, will result in undefined behavior if a value placed
// in it does not have an enumerator. Therefore, we're forced to
// do this. It's stupid. But it should work.

pub mod chunk_types {
    pub const DXT1: u16 = 0x10;
    pub const DXT1_ALT: u16 = 0x20;
    pub const FULL_NAME: u16 = 0x70;
}

/// The header of a SSSF chunk.
#[derive(Debug)]
#[repr(C)]
pub struct SssfChunkHeader {
    /// Offset to reach the start of the next chunk.
    pub next: u16,
    /// The chunk type. See the [chunk_types] module
    /// for allowed/known values of this member.
    pub chunk_type: u16,
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
    pub padding: [u16; 7],
}

#[repr(C)]
pub struct SssfFullNameChunk {
    pub unk_padding: u16,
    /// The name of this chunk's frame.
    pub name: [u8; 0xc],
}

/// The [squish::Format] that all SSSF files are encoded as.
pub const SSSF_FORMAT : squish::Format = squish::Format::Bc1;