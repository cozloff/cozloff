use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;

mod audio_log;
mod audio_source;
mod iouring_abi;
mod uring;
mod writer;

fn main() {
    if let Err(err) = run_audio_adc_demo() {
        eprintln!("audio ADC write demo failed: {err}");
    }
}

fn run_audio_adc_demo() -> io::Result<()> {
    const STREAM_ID: u64 = 1;
    const SAMPLE_RATE_HZ: u32 = 48_000;
    const CHANNELS: u16 = 2;
    const FRAMES_PER_BLOCK: u32 = 65_536;
    const BLOCK_COUNT: u64 = 8;
    const ALSA_DEVICE: &str = "default";

    let ring = uring::IoUring::new(256)?;
    let output = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(audio_log::FILE_NAME)?;

    let spec = audio_log::AudioStreamSpec {
        stream_id: STREAM_ID,
        sample_rate_hz: SAMPLE_RATE_HZ,
        channel_count: CHANNELS,
        sample_format: audio_log::SampleFormat::S16Le,
    };
    let source = audio_source::AlsaPcmCapture::new(
        STREAM_ID,
        ALSA_DEVICE,
        SAMPLE_RATE_HZ,
        CHANNELS,
        FRAMES_PER_BLOCK,
    )?;
    let buffers = std::iter::once(Ok(audio_log::file_header(spec))).chain(
        source
            .take(BLOCK_COUNT as usize)
            .map(|block| block.map(|block| audio_log::encode_block(&block))),
    );
    let stats =
        writer::write_sequential_buffers(&ring, output.as_raw_fd(), buffers, ring.capacity())?;

    println!(
        "wrote {} ADC audio buffers, {} bytes, {} errors to {}",
        stats.records,
        stats.bytes,
        stats.errors,
        audio_log::FILE_NAME
    );

    Ok(())
}
