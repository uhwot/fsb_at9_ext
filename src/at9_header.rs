use binrw::binwrite;

// https://github.com/Thealexbarney/VGAudio/blob/master/docs/audio-formats/atrac9/container.md
// based on https://github.com/itsmeft24/rainbow-ima-adpcm/blob/588093da23937edff7405c1011bcfd9ec0b8218a/src/wav.rs

#[binwrite]
#[derive(Debug)]
#[bw(little, magic = b"RIFF")]
pub struct Atrac9Header {
    pub size_remaining: u32,
    pub wave: [u8; 4],
    pub fmt: FmtChunk,
    pub fact: FactChunk,
    pub data: DataChunk,
}

#[binwrite]
#[derive(Debug)]
#[bw(magic = b"fmt ")]
pub struct FmtChunk {
    pub len: u32,
    pub format_tag: u16,
    pub channel_count: u16,
    pub sampling_rate: u32,
    pub bytes_per_second: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,

    pub extension_size: u16,
    pub samples_per_block: u16,
    pub channel_mask: u32,
    pub subformat_guid: [u8; 16],
    pub version: u32,
    #[bw(pad_after = 0x4)]
    pub at9_config: [u8; 4],
}

#[binwrite]
#[derive(Debug)]
#[bw(magic = b"fact")]
pub struct FactChunk {
    pub len: u32,
    pub num_samples: u32,
    pub input_and_overlap_delay_samples: u32,
    pub encoder_delay_samples: u32,
}

#[binwrite]
#[derive(Debug)]
#[bw(magic = b"data")]
pub struct DataChunk {
    pub len: u32,
}