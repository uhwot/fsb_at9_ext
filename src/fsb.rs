use binrw::{BinRead, binread, BinResult, parser, NullString};
use bilge::prelude::*;

#[binread]
#[derive(Debug)]
#[br(little, magic = b"FSB5")]
#[br(assert([0x0, 0x1].contains(&version)))]
pub struct Fsb {
    pub version: u32,
    #[br(temp)]
    num_samples: u32,
    pub sample_header_size: u32,
    pub name_table_size: u32,
    pub sample_data_size: u32,
    pub codec: Codec,
    #[br(pad_before = match version { 0x0 => 0x8, _ => 0x4 })]
    pub flags: [u8; 4],
    pub guid: [u8; 16],
    pub hash: [u8; 8],

    #[br(count = num_samples)]
    pub samples: Vec<Sample>,

    #[br(if(name_table_size > 0))]
    #[br(pad_before = 0x4 * num_samples)]
    #[br(count = num_samples)]
    pub names: Vec<NullString>
}

#[derive(Debug, BinRead)]
#[br(repr(u32))]
pub enum Codec {
    //None = 0x0,
    Pcm8 = 0x1,
    Pcm16 = 0x2,
    Pcm24 = 0x3,
    Pcm32 = 0x4,
    PcmFloat = 0x5,
    GcAdpcm = 0x6,
    ImaAdpcm = 0x7,
    Vag = 0x8,
    HeVag = 0x9,
    Xma = 0xa,
    Mpeg = 0xb,
    Celt = 0xc,
    At9 = 0xd,
    XWma = 0xe,
    Vorbis = 0xf,
    FAdpcm = 0x10,
    Opus = 0x11,
}

#[binread]
#[derive(Debug)]
pub struct Sample {
    #[br(map = |b: u64| SampleInfo::from(b))]
    pub info: SampleInfo,
    #[br(if(info.has_flags()))]
    #[br(parse_with = flag_parser)]
    pub flags: Vec<SampleFlag>,
}

//pub const CHANNEL_MAP: [u8; 4] = [1, 2, 6, 8];
//pub const SAMPLE_RATE_MAP: [u32; 11] = [4000, 8000, 11000, 11025, 16000, 22050, 24000, 32000, 44100, 48000, 96000];

#[bitsize(64)]
#[derive(DebugBits, FromBits)]
pub struct SampleInfo {
    pub has_flags: bool,
    pub sample_rate_idx: u4,
    pub channels_idx: u2,
    pub data_offset: u27,
    pub num_samples: u30,
}

#[bitsize(32)]
#[derive(DebugBits, FromBits)]
pub struct SampleFlagInfo {
    pub more: bool,
    pub size: u24,
    pub flag_type: u7,
}

#[parser(reader, endian)]
fn flag_parser() -> BinResult<Vec<SampleFlag>> {
    let mut flags = Vec::new();

    loop {
        let info = SampleFlagInfo::from(<u32>::read_options(reader, endian, ())?);
        let flag = match info.flag_type().value() {
            0x1 => SampleFlag::Channels(<_>::read_options(reader, endian, ())?),
            0x2 => SampleFlag::SampleRate(<_>::read_options(reader, endian, ())?),
            0x3 => SampleFlag::Loop {
                start: <_>::read_options(reader, endian, ())?,
                end: <_>::read_options(reader, endian, ())?,
            },
            0x9 => {
                let mut data = vec![0u8; info.size().value() as usize];
                reader.read_exact(&mut data)?;
                SampleFlag::Atrac9Config(data)
            },
            _ => {
                let mut data = vec![0u8; info.size().value() as usize];
                reader.read_exact(&mut data)?;
                SampleFlag::Unknown(data)
            },
        };
        flags.push(flag);
        if !info.more() {
            break;
        }
    }

    Ok(flags)
}

#[derive(Debug)]
pub enum SampleFlag {
    Channels(u8),
    SampleRate(u32),
    Loop { start: u32, end: u32 },
    Atrac9Config(Vec<u8>),
    Unknown(Vec<u8>),
}