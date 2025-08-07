use binext::BinaryRead;
use clap::*;
use eyre::eyre;
use std::{
    fs::File,
    io::{BufWriter, Read},
    path::{Path, PathBuf},
    process::exit,
};
use streaming_shape_tools::{ChunkParser, sss_structs::*};

// this is really stupid, but it works. The best kind
fn str_from_c_string(slice: &[u8]) -> Result<&str, std::str::Utf8Error> {
    let frame_name_raw = std::str::from_utf8(&slice[..])?;
    // strip off everything after nul. but if there's no nul
    // just leave it be
    if let Some(off) = frame_name_raw.find('\0') {
        Ok(&frame_name_raw[0..off])
    } else {
        Ok(frame_name_raw)
    }
}

pub struct SssfData {
    pub width: u16,
    pub height: u16,
    pub data: Box<[u8]>,
    pub name: SssfFullNameChunk,
}

/// read a frame chunk and spit out the data
pub fn read_sssf_frame<T: Read>(mut read: T) -> eyre::Result<SssfData> {
    let mut image_header: Option<SssfFrameChunkHeader> = None;
    let mut image_data: Option<Vec<u8>> = None;

    loop {
        let chunk_header = read.read_binary::<SssfChunkHeader>()?;
        match chunk_header.chunk_type {
            chunk_types::DXT1 | chunk_types::DXT1_ALT => {
                image_header = Some(read.read_binary::<SssfFrameChunkHeader>()?);
                let hdr_ref = image_header.as_ref().unwrap();
                // Read the data
                image_data = Some(vec![
                    0;
                    SSSF_FORMAT.compressed_size(
                        hdr_ref.width as usize,
                        hdr_ref.height as usize
                    )
                ]);
                read.read_exact(&mut image_data.as_mut().unwrap()[..])?;

                let _footer_discarded = read.read_binary::<SssfImageChunkFooter>()?;
            }

            chunk_types::FULL_NAME => {
                // A full name chunk terminates the chunk
                let hdr = image_header.as_ref().unwrap();
                return Ok(SssfData {
                    width: hdr.width,
                    height: hdr.height,
                    data: image_data.expect("???").into_boxed_slice(),
                    name: read.read_binary::<SssfFullNameChunk>()?,
                });
            }

            // idiot checking
            _ => {
                panic!(
                    "unhandled chunk type {:02x} in frame stream chunk data.",
                    chunk_header.chunk_type
                );
            }
        }
    }
}

fn export_frame<P: AsRef<Path>>(
    basename: P,
    frame: &SssfData,
    frame_index: usize,
) -> eyre::Result<()> {
    let frame_name = str_from_c_string(&frame.name.name)?;
    let mut frame_buffer: Vec<u8> = vec![0; frame.width as usize * frame.height as usize * 4usize];

    // Decompress the DXT1 compressed data
    SSSF_FORMAT.decompress(
        &frame.data[..],
        frame.width as usize,
        frame.height as usize,
        &mut frame_buffer[..],
    );

    let mut image_path = basename.as_ref().to_owned();
    image_path.push(format!("{frame_name}_{frame_index}.png"));
    let file = File::create(&image_path)?;

    // Truncate any existing file. This isn't *strictly* needed,
    // but still good practice to do.
    file.set_len(0)?;

    let mut bufwriter = BufWriter::new(file);

    image::write_buffer_with_format(
        &mut bufwriter,
        &frame_buffer[..],
        frame.width as u32,
        frame.height as u32,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;

    drop(bufwriter);
    println!("Wrote frame {frame_index} to {}", image_path.display());

    Ok(())
}

fn main() -> eyre::Result<()> {
    let matches = Command::new("extract_sssf")
        .about(crate_description!())
        .arg(
            Arg::new("file")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    let path = matches.get_one::<PathBuf>("file").unwrap();

    if !path.is_file() {
        return Err(eyre!("Input file {} is not an actual file", path.display()));
    }

    let mut output_path = path.parent().unwrap().to_owned();
    let new_folder_name = format!("{}_extracted", path.file_stem().unwrap().to_str().unwrap());
    output_path.push(new_folder_name);

    std::fs::create_dir_all(&output_path)?;

    let file = File::open(path)?;
    let mut parser = ChunkParser::new(file);
    let mut frame_index: usize = 0;
    for chunk in &mut parser {
        let data = read_sssf_frame(std::io::Cursor::new(&chunk.data[..]))?;
        export_frame(&output_path, &data, frame_index)?;
        frame_index += 1;
    }

    Ok(())
}
