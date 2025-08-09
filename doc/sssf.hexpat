import std.mem;
import std.io;
import std.math;

struct ea_iff_chunk {
    u32 ctype;
    u32 csize;
};

fn get_dxt1_length(u32 width, u32 height) {
    u32 hblock = std::math::ceil(width / 4);
    u32 vblock = std::math::ceil(height / 4);
    return (hblock*vblock)*8;
};

struct sss_shape_chunk {
    u8 type;
    u24 next;
};

// most of these "unk"s are probably padding.
struct sss_image_chunk {
    u16 width;
    u16 height;
    u16 unk;
    u16 unk2;
    u16 unk3;
    u16 unk4;
    
    u8 data[get_dxt1_length(width, height)];
    
    u16 unk5;
    u16 unk6;
    u16 unk7;
    u16 unk8;
    u16 unk9;
    u16 unka;
    u16 unkb;
    u16 unkc;
};

struct sss_fullname_chunk {
    char name[0xc];
};

struct sss_chunk_dispatch {
    sss_shape_chunk c;
    if(c.type == 0x60) {
        sss_image_chunk image;
    } else if(c.type == 0x70) {
        sss_fullname_chunk name;
    }
};

struct sssf_chunk {
     sss_chunk_dispatch image;
     sss_chunk_dispatch fullname;
};

struct iff_chunk_dispatch {
    ea_iff_chunk c;
    if(c.ctype == 0x46535353) // 'SSSF'
        sssf_chunk sssf;
};

iff_chunk_dispatch chunk[while(!std::mem::eof())] @ 0;