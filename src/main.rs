use std::{env::args, path::Path, fs::{File, self}, io::{Write, Seek, SeekFrom, Read}};

use binrw::{BinRead, BinWrite};
use fsb::{Fsb, Codec};

use crate::{fsb::SampleFlag, at9_config::At9Config, at9_header::*};

mod fsb;
mod at9_header;
mod at9_config;

fn find_at9_config(flags: &Vec<SampleFlag>) -> &Vec<u8> {
    for flag in flags {
        if let SampleFlag::Atrac9Config(config) = flag {
            return config;
        }
    }
    panic!("ATRAC9 config not found");
}

fn bytes_to_align(pos: u64, align_num: u64) -> u64 {
    if pos % align_num != 0 {
        align_num - pos % align_num
    } else {
        0
    }
}

fn create_out_file(name: &str, size: u32, num_samples: u32, is_multichannel: bool, i: usize, at9_config: [u8; 4]) -> (File, u16) {
    let mut filename = name.to_string();
    if is_multichannel {
        filename = format!("{filename}/{}", i+1);
    }
    filename += ".at9";

    let config = At9Config::parse(at9_config);

    println!("filename: {filename}");
    println!("size: {size}");
    println!("sample count: {}", num_samples);

    let superframe_bytes = config.frame_bytes * config.frames_per_superframe;
    let superframe_samples = config.frame_samples * config.frames_per_superframe;

    let bitrate = superframe_bytes as u32 * config.sample_rate / superframe_samples as u32;

    println!("{config:#?}");
    println!("channel mask: {:b}", config.channel_mask);
    println!("superframe bytes: {superframe_bytes}");
    println!("superframe samples: {superframe_samples}");
    println!("bitrate: {}kbps", bitrate / 125);
    println!();

    let header = Atrac9Header {
        size_remaining: size + 0x5C,
        fmt: FmtChunk {
            len: 52,
            format_tag: 0xFFFE,
            channel_count: config.num_channels.into(),
            sampling_rate: config.sample_rate,
            bytes_per_second: bitrate,
            block_align: superframe_bytes,
            bits_per_sample: 0,

            extension_size: 34,
            samples_per_block: superframe_samples,
            channel_mask: config.channel_mask.into(),
            subformat_guid: [0xD2, 0x42, 0xE1, 0x47, 0xBA, 0x36, 0x8D, 0x4D, 0x88, 0xFC, 0x61, 0x65, 0x4F, 0x8C, 0x83, 0x6C],
            version: 1,
            at9_config,
        },
        fact: FactChunk {
            len: 12,
            num_samples,
            input_and_overlap_delay_samples: 256,
            encoder_delay_samples: 256,
        },
        data: DataChunk {
            len: size
        },
    };

    let mut out = File::create(filename).unwrap();
    header.write(&mut out).expect("Couldn't write ATRAC9 header");

    (out, superframe_bytes)
}

fn main() {
    let mut args = args();
    args.next();
    let fsb_path = args.next().expect("FSB path not in arguments");
    let fsb_path = Path::new(&fsb_path);

    let mut file = File::open(fsb_path).expect("Couldn't open FSB");
    let fsb = Fsb::read(&mut file).expect("Couldn't parse FSB");

    //println!("{fsb:#x?}");

    match fsb.codec {
        Codec::At9 => {},
        _ => {
            println!("FSB doesn't contain ATRAC9 samples, quitting");
            return
        }
    }

    // sample data is always aligned to 32 bytes
    let sample_data_align = bytes_to_align(file.stream_position().unwrap(), 32);
    file.seek(SeekFrom::Current(sample_data_align as i64)).unwrap();

    for (i, sample) in fsb.samples.iter().enumerate() {
        let name = fsb.names.get(i)
            .map(|s| s.to_string())
            .unwrap_or_else(|| (i+1).to_string());

        let size = match fsb.samples.get(i + 1) {
            Some(next) => (next.info.data_offset().value() - sample.info.data_offset().value()) * 32,
            None => fsb.sample_data_size - sample.info.data_offset().value() * 32,
        };

        println!("sample name: {name}");

        let mut at9_configs = find_at9_config(&sample.flags).as_slice();
        if at9_configs[0] != 0xFE {
            at9_configs = &at9_configs[4..];
        }
        let num_files = at9_configs.len() / 4;

        let is_multichannel = num_files > 1;
        if is_multichannel {
            fs::create_dir_all(&name).unwrap();
        }

        let at9_configs = at9_configs.chunks(4);
        let mut out_files: Vec<(File, u16)> = at9_configs
            .enumerate()
            .map(|(i, c)| create_out_file(
                &name,
                size / num_files as u32,
                sample.info.num_samples().value(),
                is_multichannel,
                i,
                c.try_into().unwrap(),
            ))
            .collect();

        let mut handle = (&mut file).take(size.into());

        let mut i = 0;
        while handle.limit() != 0 {
            let (ref mut out, superframe_bytes) = &mut out_files[i % num_files];
            let mut superframe_data = vec![0u8; *superframe_bytes as usize];
            handle.read_exact(&mut superframe_data).unwrap();
            out.write_all(&superframe_data).unwrap();
            i = (i+1) % num_files;
        }
    }
}
