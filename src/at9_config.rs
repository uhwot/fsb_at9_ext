// https://github.com/Thealexbarney/VGAudio/blob/master/docs/audio-formats/atrac9/container.md#config-data

const SAMPLE_RATE_MAP: [u32; 16] = [
    11025, 12000, 16000, 22050, 24000, 32000, 44100, 48000,
    44100, 48000, 64000, 88200, 96000, 128000, 176400, 192000
];

const CHANNEL_MAP: [u8; 6] = [1, 2, 2, 6, 8, 4];

// https://www.psdevwiki.com/ps4/Template:Wave_Channel_Mask
const CHANNEL_MASK_MAP: [u8; 6] = [0b00000100, 0b00000100, 0b00000011, 0b11001111, 0b11111111, 0b00110011];

const FRAME_SAMPLE_POWER_MAP: [u8; 16] = [6, 6, 7, 7, 7, 8, 8, 8, 6, 6, 7, 7, 7, 8, 8, 8];

#[derive(Debug)]
pub struct At9Config {
    pub sample_rate: u32,
    pub frame_samples: u16,
    pub num_channels: u8,
    pub channel_mask: u8,
    pub frame_bytes: u16,
    pub frames_per_superframe: u16,
}

impl At9Config {
    pub fn parse(data: [u8; 4]) -> Self {
        let num = u32::from_be_bytes(data);

        if (num >> 24 & 0xFF) != 0xFE {
            panic!("Invalid ATRAC9 config sync code");
        }

        if (num >> 16 & 0b1) != 0 {
            panic!("Invalid ATRAC9 config validation bit");
        }

        let sample_rate_idx = (num >> 20 & 0b1111) as usize;
        let channel_cfg_idx = (num >> 17 & 0b111) as usize;

        Self {
            sample_rate: SAMPLE_RATE_MAP[sample_rate_idx],
            frame_samples: 1 << FRAME_SAMPLE_POWER_MAP[sample_rate_idx],
            num_channels: CHANNEL_MAP[channel_cfg_idx],
            channel_mask: CHANNEL_MASK_MAP[channel_cfg_idx],
            frame_bytes: ((num >> 5 & 0b11111111111) + 1).try_into().unwrap(),
            frames_per_superframe: (1 << (num >> 3 & 0b11)).try_into().unwrap(),
        }
    }
}