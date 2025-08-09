use binext::BinaryRead;
use clap::*;
use eyre::eyre;
use std::{
    fs::File,
    io::{BufWriter, Read, Seek},
    path::{Path, PathBuf},
};
use streaming_shape_tools::{
    ChunkParser,
    manifest::{ManifestFrame, ManifestRoot},
    sss_structs::*,
};

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
    pub name: Option<String>,
}

/// read a frame chunk and spit out the data
pub fn read_sssf_frame<T: Read + Seek>(mut read: T) -> eyre::Result<SssfData> {
    let mut frame_header: Option<SssfFrameChunkHeader> = None;
    let mut frame_name: Option<String> = None;
    let mut frame_compressed_data: Option<Vec<u8>> = None;

    // FIXME: I'm very not happy with this code.
    // A newtype chunk iterator like the IFF code (that just spits out chunks + unparsed data,
    // and expects the client to do the parsing) should be possible to write. But this should work...
    let mut chunk_header: SssfChunkHeader = read.read_binary::<SssfChunkHeader>()?;
    loop {
        let current_pos = read.seek(std::io::SeekFrom::Current(0))? as u32;
        let next_pos = (current_pos - 4) + chunk_header.next();

        match chunk_header.chunk_type {
            chunk_types::DXT1 => {
                frame_header = Some(read.read_binary::<SssfFrameChunkHeader>()?);
                let hdr_ref = frame_header.as_ref().unwrap();
                // Read the data
                frame_compressed_data = Some(vec![
                    0;
                    SSSF_FORMAT.compressed_size(
                        hdr_ref.width as usize,
                        hdr_ref.height as usize
                    )
                ]);
                read.read_exact(&mut frame_compressed_data.as_mut().unwrap()[..])?;
            }

            chunk_types::FULL_NAME => {
                let full_name_chunk = read.read_binary::<SssfFullNameChunk>()?;
                frame_name = Some(str_from_c_string(&full_name_chunk.name)?.to_string());
            }

            // idiot checking
            _ => {
                panic!(
                    "unhandled chunk type {:02x} in frame stream chunk data.",
                    chunk_header.chunk_type
                );
            }
        }

        // There are no more chunks.
        if chunk_header.next() == 0 {
            break;
        }

        // Seek to and read the next chunk header, so we can parse the data
        // in the next iteration of this loop.
        read.seek(std::io::SeekFrom::Start(next_pos as u64))?;
        chunk_header = read.read_binary::<SssfChunkHeader>()?;
    }

    let hdr = frame_header.as_ref().unwrap();
    return Ok(SssfData {
        width: hdr.width,
        height: hdr.height,
        data: frame_compressed_data.expect("???").into_boxed_slice(),
        name: frame_name,
    });
}

fn export_frame<P: AsRef<Path>>(
    output_path: P,
    frame: &SssfData,
    frame_index: usize,
    frames: &mut Vec<ManifestFrame>,
) -> eyre::Result<()> {
    let frame_name = frame
        .name
        .as_ref()
        .expect("no frame name provided, the code can't currently handle this correctly");
    let mut frame_buffer: Vec<u8> = vec![0; frame.width as usize * frame.height as usize * 4usize];

    // Decompress the DXT1 compressed data
    SSSF_FORMAT.decompress(
        &frame.data[..],
        frame.width as usize,
        frame.height as usize,
        &mut frame_buffer[..],
    );

    let mut image_path = output_path.as_ref().to_owned();
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

    frames.push(ManifestFrame {
        path: image_path.canonicalize().unwrap().to_str().unwrap().into(),
        frame_name: Some(frame_name.into()),
    });

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

    let output_frame_path = output_path.join("frames");
    let output_manifest_path = output_path.join("ssf.json");

    std::fs::create_dir_all(&output_path)?;
    std::fs::create_dir_all(&output_frame_path)?;

    let file = File::open(path)?;
    let mut frame_index: usize = 0;

    let mut manifest = ManifestRoot {
        width: 0,
        height: 0,
        frames: Vec::new(),
    };

    for chunk in ChunkParser::new(file) {
        let data = read_sssf_frame(std::io::Cursor::new(&chunk.data[..]))?;
        if manifest.width == 0 {
            manifest.width = data.width as u32;
            manifest.height = data.height as u32;
        }

        export_frame(&output_frame_path, &data, frame_index, &mut manifest.frames)?;
        frame_index += 1;
    }

    let file = File::create(&output_manifest_path)?;
    serde_json::to_writer_pretty(file, &manifest)?;
    println!("manifest written to {}", output_manifest_path.display());

    Ok(())
}
