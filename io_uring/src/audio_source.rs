use crate::audio_log::AudioBlock;
use std::io::{self, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AlsaPcmCapture {
    stream_id: u64,
    channel_count: u16,
    frames_per_block: u32,
    child: Child,
    stdout: ChildStdout,
    next_frame: u64,
    next_block: u64,
}

impl AlsaPcmCapture {
    pub fn new(
        stream_id: u64,
        device: &str,
        sample_rate_hz: u32,
        channel_count: u16,
        frames_per_block: u32,
    ) -> io::Result<Self> {
        ensure_capture_device_available()?;

        let mut child = Command::new("arecord")
            .args([
                "-q",
                "-D",
                device,
                "-f",
                "S16_LE",
                "-r",
                &sample_rate_hz.to_string(),
                "-c",
                &channel_count.to_string(),
                "-t",
                "raw",
            ])
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::BrokenPipe, "arecord stdout was not captured")
        })?;

        Ok(Self {
            stream_id,
            channel_count,
            frames_per_block,
            child,
            stdout,
            next_frame: 0,
            next_block: 0,
        })
    }
}

impl Iterator for AlsaPcmCapture {
    type Item = io::Result<AudioBlock>;

    fn next(&mut self) -> Option<io::Result<AudioBlock>> {
        let bytes_per_sample = 2usize;
        let block_len =
            self.frames_per_block as usize * self.channel_count as usize * bytes_per_sample;
        let mut payload = vec![0u8; block_len];

        let mut read = 0usize;
        while read < payload.len() {
            match self.stdout.read(&mut payload[read..]) {
                Ok(0) if read == 0 => return None,
                Ok(0) => {
                    payload.truncate(read);
                    break;
                }
                Ok(n) => read += n,
                Err(err) => return Some(Err(err)),
            }
        }

        let block = AudioBlock {
            stream_id: self.stream_id,
            block_sequence: self.next_block,
            start_frame: self.next_frame,
            timestamp_ns: now_ns(),
            frame_count: (payload.len() / (self.channel_count as usize * bytes_per_sample)) as u32,
            payload,
        };
        self.next_frame += block.frame_count as u64;
        self.next_block += 1;
        Some(Ok(block))
    }
}

impl Drop for AlsaPcmCapture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn ensure_capture_device_available() -> io::Result<()> {
    let output = Command::new("arecord").arg("-l").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if output.status.success()
        && stdout
            .lines()
            .any(|line| line.trim_start().starts_with("card "))
    {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no ALSA capture devices found by `arecord -l`",
        ))
    }
}

fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
