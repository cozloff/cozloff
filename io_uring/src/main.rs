use std::ffi::c_void;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

pub mod iouring_abi;

/// Pointers to the shared memory ring mappings
/// 
/// Kernel responses to tail to consume in user space via shared memory.
pub struct CompletionQueue {
    pub head: *const AtomicU32,  // Points to kernel-shared head
    pub tail: *const AtomicU32,  // Points to kernel-shared tail
    pub ring_mask: u32,          // (Queue capacity - 1) used for bitwise modulo
    pub cqes: *const iouring_abi::io_uring_cqe, // Array of CQ Entries
}
// User space tail async requests to kernel in shared memory
struct SubmissionQueue {
    tail: *const AtomicU32, // Points to kernel-shared tail
    ring_mask: u32,         // (Queue capacity - 1) used for bitwise modulo
    array: *mut u32,        // Array of submission queue indices
    sqes: *mut iouring_abi::io_uring_sqe, // Array of SQ Entries
}

fn main() {
    unsafe {
        // Send io_uring setup to kernel and get the file descriptor and params.
        //
        // File descripter - used in all interactions w/ the 
        //                   kernel for io_uring instance
        let mut params = iouring_abi::io_uring_params::default();
        let fd = iouring_abi::syscall( 
            iouring_abi::SYS_IO_URING_SETUP, 
            8u32, 
            &mut params) as i32;

        if fd < 0 {
            eprintln!(
                "io_uring_setup failed: {}", 
                std::io::Error::last_os_error());
            return;
        }

        // Offset + Size (entries * 4 bytes per index)
        let sq_ring_size = params.sq_off.array as usize + (
            params.sq_entries as usize * 4);
        // Offset + Size (entries * size_of::<CQE>)
        let cq_ring_size = params.cq_off.cqes as usize + (
                params.cq_entries as usize * 
                mem::size_of::<iouring_abi::io_uring_cqe>());
        // Entries * size_of::<SQE>
        let sqes_size = params.sq_entries as usize * 
            mem::size_of::<iouring_abi::io_uring_sqe>();

        let sq_ring = iouring_abi::mmap(
            ptr::null_mut(),
            sq_ring_size,
            iouring_abi::PROT_READ | iouring_abi::PROT_WRITE,
            iouring_abi::MAP_SHARED,
            fd,
            iouring_abi::IORING_OFF_SQ_RING,
        );
        let cq_ring = iouring_abi::mmap(
            ptr::null_mut(),
            cq_ring_size,
            iouring_abi::PROT_READ | iouring_abi::PROT_WRITE,
            iouring_abi::MAP_SHARED,
            fd,
            iouring_abi::IORING_OFF_CQ_RING,
        );
        let sqes = iouring_abi::mmap(
            ptr::null_mut(),
            sqes_size,
            iouring_abi::PROT_READ | iouring_abi::PROT_WRITE,
            iouring_abi::MAP_SHARED,
            fd,
            iouring_abi::IORING_OFF_SQES,
        );

        if sq_ring == iouring_abi::MAP_FAILED || cq_ring == iouring_abi::MAP_FAILED || sqes == iouring_abi::MAP_FAILED {
            eprintln!("iouring_abi::mmap failed");
            return;
        }

        let sq = SubmissionQueue {
            tail: sq_ring.byte_add(params.sq_off.tail as usize) as *const AtomicU32,
            ring_mask: *(sq_ring.byte_add(params.sq_off.ring_mask as usize) as *const u32),
            array: sq_ring.byte_add(params.sq_off.array as usize) as *mut u32,
            sqes: sqes as *mut iouring_abi::io_uring_sqe,
        };

        let cq = CompletionQueue {
            head: cq_ring.byte_add(params.cq_off.head as usize) as *const AtomicU32,
            tail: cq_ring.byte_add(params.cq_off.tail as usize) as *const AtomicU32,
            ring_mask: *(cq_ring.byte_add(params.cq_off.ring_mask as usize) as *const u32),
            cqes: cq_ring.byte_add(params.cq_off.cqes as usize) as *const iouring_abi::io_uring_cqe,
        };

        // Send No Operation to kernel to test submission and completion
        submit_nop(fd, &sq); 

        // Check for completions and print stats
        process_completions(&cq);
    }
}

// Submits a single NOP operation to the kernel via the submission queue.
unsafe fn submit_nop(fd: i32, sq: &SubmissionQueue) {
    let tail = unsafe { (*sq.tail).load(Ordering::Relaxed) };
    let index = tail & sq.ring_mask;
    let sqe = unsafe { &mut *sq.sqes.add(index as usize) };

    *sqe = iouring_abi::io_uring_sqe::default();
    sqe.opcode = 0; // IORING_OP_NOP
    sqe.user_data = 42;

    unsafe {
        *sq.array.add(index as usize) = index;
        (*sq.tail).store(tail.wrapping_add(1), Ordering::Release);
        iouring_abi::syscall(
            iouring_abi::SYS_IO_URING_ENTER,
            fd,
            1u32,
            1u32,
            iouring_abi::IORING_ENTER_GETEVENTS,
            ptr::null::<c_void>(),
            0usize,
        );
    }
}

/// Provides peek and advance methods for the completion queue to read kernel responses.
impl CompletionQueue {
    /// Peeks at the queue. Returns a slice of available CQEs and their starting head index.
    /// Does not advance the queue or block.
    pub unsafe fn peek(&self) -> Option<(&[iouring_abi::io_uring_cqe], u32)> {
        // Read head and tail with strict atomic ordering
        let head = unsafe { (*self.head).load(Ordering::Relaxed) };
        let tail = unsafe { (*self.tail).load(Ordering::Acquire) }; // Acquire ensures we see kernel writes

        if head == tail {
            return None; // Queue is empty
        }

        // For simplicity in raw pointer manipulation, we read 1 item.
        // (Handling a batch requires looping and accounting for ring-buffer wrap-around).
        let index = (head & self.ring_mask) as usize;
        let cqe_ptr = unsafe { self.cqes.add(index) };
        
        Some((unsafe { std::slice::from_raw_parts(cqe_ptr, 1) }, head))
    }

    /// Advances the head pointer to notify the kernel that you consumed the entries.
    pub unsafe fn advance(&self, last_head: u32, count: u32) {
        let new_head = last_head.wrapping_add(count);
        // Release ordering ensures the kernel sees we are done reading the memory
        unsafe {
            (*self.head).store(new_head, Ordering::Release);
        }
    }
}

// Processes available completions in the completion queue and prints stats on successes and errors.
fn process_completions(cq: &CompletionQueue) {
    unsafe {
        let mut completions = 0;
        let mut successes = 0;
        let mut errors = 0;

        // Check if anything is ready
        if let Some((cqes, item_head)) = cq.peek() {
            for cqe in cqes {
                completions += 1;

                if cqe.res < 0 {
                    errors += 1;
                } else {
                    successes += 1;
                }
            }

            // Mark 1 item as completed so the kernel can reuse the slot
            cq.advance(item_head, 1);
        }

        println!(
            "CQ stats: completions={completions}, successes={successes}, errors={errors}"
        );
    }
}
