pub const FILE_NAME: &str = "audio_adc.adclog";

const FILE_MAGIC: &[u8; 8] = b"ADCLOG1\n";
const BLOCK_MAGIC: u32 = 0x4144_4331; // ADC1
const FILE_HEADER_LEN: u16 = 64;
const BLOCK_HEADER_LEN: u16 = 64;

#[derive(Clone, Copy)]
pub enum SampleFormat {
    S16Le,
}

pub struct AudioStreamSpec {
    pub stream_id: u64,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub sample_format: SampleFormat,
}

pub struct AudioBlock {
    pub stream_id: u64,
    pub block_sequence: u64,
    pub start_frame: u64,
    pub timestamp_ns: u64,
    pub frame_count: u32,
    pub payload: Vec<u8>,
}

pub fn file_header(spec: AudioStreamSpec) -> Vec<u8> {
    let mut out = vec![0u8; FILE_HEADER_LEN as usize];
    out[0..8].copy_from_slice(FILE_MAGIC);
    out[8..10].copy_from_slice(&FILE_HEADER_LEN.to_le_bytes());
    out[10..12].copy_from_slice(&1u16.to_le_bytes());
    out[16..24].copy_from_slice(&spec.stream_id.to_le_bytes());
    out[24..28].copy_from_slice(&spec.sample_rate_hz.to_le_bytes());
    out[28..30].copy_from_slice(&spec.channel_count.to_le_bytes());
    out[30..32].copy_from_slice(&sample_format_id(spec.sample_format).to_le_bytes());
    out
}

pub fn encode_block(block: &AudioBlock) -> Vec<u8> {
    let mut out = Vec::with_capacity(BLOCK_HEADER_LEN as usize + block.payload.len());
    out.extend_from_slice(&BLOCK_MAGIC.to_le_bytes());
    out.extend_from_slice(&BLOCK_HEADER_LEN.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&block.stream_id.to_le_bytes());
    out.extend_from_slice(&block.block_sequence.to_le_bytes());
    out.extend_from_slice(&block.start_frame.to_le_bytes());
    out.extend_from_slice(&block.timestamp_ns.to_le_bytes());
    out.extend_from_slice(&block.frame_count.to_le_bytes());
    out.extend_from_slice(&(block.payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&checksum(&block.payload).to_le_bytes());
    out.resize(BLOCK_HEADER_LEN as usize, 0);
    out.extend_from_slice(&block.payload);
    out
}

fn sample_format_id(format: SampleFormat) -> u16 {
    match format {
        SampleFormat::S16Le => 1,
    }
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |sum, byte| sum.wrapping_add(*byte as u32))
}
