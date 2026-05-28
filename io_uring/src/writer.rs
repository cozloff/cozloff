use crate::uring::IoUring;
use std::io;
use std::os::fd::RawFd;

pub struct WriteStats {
    pub records: u64,
    pub bytes: u64,
    pub errors: u64,
}

pub fn write_sequential_buffers<I>(
    ring: &IoUring,
    file_fd: RawFd,
    mut buffers: I,
    max_in_flight: u32,
) -> io::Result<WriteStats>
where
    I: Iterator<Item = io::Result<Vec<u8>>>,
{
    let mut in_flight: Vec<Option<Vec<u8>>> = vec![None; max_in_flight as usize];
    let mut slot_offsets = vec![0u64; max_in_flight as usize];
    let mut submitted = 0u64;
    let mut completed = 0u64;
    let mut bytes = 0u64;
    let mut next_file_offset = 0u64;
    let mut errors = 0u64;
    let mut input_done = false;

    while !input_done || completed < submitted {
        while !input_done
            && submitted - completed < max_in_flight as u64
            && ring.available_submissions() > 0
        {
            let Some(slot_id) = in_flight.iter().position(Option::is_none) else {
                break;
            };
            let Some(buffer) = buffers.next().transpose()? else {
                input_done = true;
                break;
            };

            let file_offset = next_file_offset;
            next_file_offset += buffer.len() as u64;
            bytes += buffer.len() as u64;
            slot_offsets[slot_id] = file_offset;
            in_flight[slot_id] = Some(buffer);
            let buffer_ref = in_flight[slot_id].as_ref().expect("buffer just inserted");

            // The buffer must remain at the same address until its CQE arrives. The slot pool
            // gives each in-flight SQE stable ownership and releases it only on completion.
            //
            // Explicit offsets avoid the shared file-position lock behind O_APPEND and are the
            // stepping stone to per-core segment writers that target independent NVMe queues.
            ring.submit_write(file_fd, buffer_ref, file_offset, slot_id as u64)?;
            submitted += 1;
        }

        let want = if submitted == completed { 0 } else { 1 };
        ring.wait_for_completions(want)?;

        while let Some(completion) = ring.pop_completion() {
            completed += 1;
            if completion.result < 0 {
                errors += 1;
            } else if let Some(buffer) = in_flight
                .get(completion.user_data as usize)
                .and_then(Option::as_ref)
            {
                if completion.result as usize != buffer.len() {
                    errors += 1;
                }
            }
            if let Some(slot) = in_flight.get_mut(completion.user_data as usize) {
                *slot = None;
            }
        }
    }

    Ok(WriteStats {
        records: completed,
        bytes,
        errors,
    })
}
