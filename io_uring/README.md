# High-Bandwidth Audio ADC Capture

This crate is a low-level Linux prototype for writing real audio ADC PCM data into a
block-oriented binary log with `io_uring`.

Current architecture:

- ALSA capture via `arecord` raw `S16_LE` PCM stream
- large audio blocks instead of per-sample records
- `ADCLOG1` file header plus repeated `ADC1` blocks
- explicit file offsets instead of `O_APPEND`
- stable in-flight buffers until each CQE returns
- `io_uring` write submission/completion path

Next low-level steps:

- replace `arecord` subprocess capture with direct ALSA mmap capture
- preallocate the output segment
- use aligned buffers
- register buffers and file descriptors with `io_uring`
- add periodic fsync/checkpoints
- split per-device/per-core segment writers for many-channel capture
