# fsb_at9_ext
a crappy tool written in Rust to extract ATRAC9 samples from FSB5 (FMOD Sample Bank) files\
multichannel samples are supported, while loop info isn't written to the extracted files\
tested with LittleBigPlanet Vita and Bloodborne

# building
`cargo build --release`

# usage
`./fsb_at9_ext <path to fsb>`\
samples are extracted in the current directory

# thanks to
[vgmstream](https://github.com/vgmstream/vgmstream)\
[VGAudio](https://github.com/Thealexbarney/VGAudio)\
[rainbow-ima-adpcm](https://github.com/itsmeft24/rainbow-ima-adpcm)