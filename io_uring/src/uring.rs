use std::io;
use std::mem;
use std::ptr;
use std::ffi::c_void;
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::iouring_abi;

pub struct IoUring {
    // File descriptor for io_uring instance
    fd: RawFd,
    // Submission queue shared with the kernel
    sq: SubmissionQueue,
    // Completion queue shared with the kernel
    cq: CompletionQueue,
    // Number of entries in the submission queue
    sq_entries: u32, // 32K max
}

pub struct Completion {
    // User data passed in SQE that triggered this completion.
    pub user_data: u64, 
    // Result code from the kernel for this completion.
    pub result: i32,
}

struct SubmissionQueue {
    head: *const AtomicU32,
    tail: *const AtomicU32,
    ring_mask: u32,
    ring_entries: u32,
    array: *mut u32,
    sqes: *mut iouring_abi::io_uring_sqe,
}

struct CompletionQueue {
    head: *const AtomicU32,
    tail: *const AtomicU32,
    ring_mask: u32,
    cqes: *const iouring_abi::io_uring_cqe,
}

impl IoUring {
    /// Create a new io_uring instance with the specified number 
    /// of SQ entries.
    pub fn new(entries: u32) -> io::Result<Self> {
        /// Init io_uring instance, get file descriptor and params.
        let (fd, params) = Self::init_io_uring()?;

        /// Map SQ/CQ rings from kernel using the fd and offsets.
        let sq_ring = Self::get_sq_ring(&params, fd)?;
        let cq_ring = Self::get_cq_ring(&params, fd)?;
        let sqes = mmap_ring(fd, sqes_size, iouring_abi::IORING_OFF_SQES)?;
        let sqes_size = params.sq_entries as usize * 
            mem::size_of::<iouring_abi::io_uring_sqe>();

        /// These pointers refer to kernel-shared mmap regions. 
        /// The mapping must outlive all submission/completion access; 
        /// this prototype owns the process lifetime and does not
        /// unmap/close yet because the next step is to wrap the 
        /// mappings in Drop-aware handles.
        let sq = unsafe {
            SubmissionQueue {
                head: sq_ring.byte_add(
                    params.sq_off.head as usize) as *const AtomicU32,
                tail: sq_ring.byte_add(
                    params.sq_off.tail as usize) as *const AtomicU32,
                ring_mask: *(sq_ring.byte_add(
                    params.sq_off.ring_mask as usize) as *const u32),
                ring_entries: *(sq_ring.byte_add(
                    params.sq_off.ring_entries as usize) as *const u32),
                array: sq_ring.byte_add(
                    params.sq_off.array as usize) as *mut u32,
                sqes: sqes as *mut iouring_abi::io_uring_sqe,
            }
        };
        let cq = unsafe {
            CompletionQueue {
                head: cq_ring.byte_add(
                    params.cq_off.head as usize) as *const AtomicU32,
                tail: cq_ring.byte_add(
                    params.cq_off.tail as usize) as *const AtomicU32,
                ring_mask: *(cq_ring.byte_add(
                    params.cq_off.ring_mask as usize) as *const u32),
                cqes: cq_ring.byte_add(
                    params.cq_off.cqes as usize) as *const iouring_abi::io_uring_cqe,
            }
        };

        Ok(Self {
            fd,
            sq,
            cq,
            sq_entries: params.sq_entries,
        })
    }

    /// Capacity of the SQ in-flight
    pub fn capacity(&self) -> u32 {
        self.sq_entries
    }

    /// Max SQE submitted w/o overflowing the SQ ring.
    pub fn available_submissions(&self) -> u32 {
        self.sq.available()
    }

    /// Prep write op in next available SQE slot -> submit to kernel.
    pub fn submit_write(
        &self,
        file_fd: RawFd,
        buffer: &[u8],
        file_offset: u64,
        user_data: u64,
    ) -> io::Result<()> {
        self.sq
            .prepare_write(file_fd, buffer, file_offset, user_data);
        self.enter(1, 0)
    }

    /// Wait for at least `min_complete` completions.
    pub fn wait_for_completions(&self, min_complete: u32) -> io::Result<()> {
        self.enter(0, min_complete)
    }

    /// Pop a single completion from the CQ, if available.
    pub fn pop_completion(&self) -> Option<Completion> {
        let (cqe, head) = self.cq.peek_one()?;
        self.cq.advance(head, 1);
        Some(Completion {
            user_data: cqe.user_data,
            result: cqe.res,
        })
    }

    /// Call `io_uring_enter` syscall to submit SQEs and/or wait for CQEs.
    fn enter(&self, to_submit: u32, min_complete: u32) -> io::Result<()> {
        let flags = if min_complete > 0 {
            iouring_abi::IORING_ENTER_GETEVENTS
        } else {
            0
        };
        let result = unsafe {
            iouring_abi::syscall(
                iouring_abi::SYS_IO_URING_ENTER,
                self.fd,
                to_submit,
                min_complete,
                flags,
                ptr::null::<c_void>(),
                0usize,
            )
        };

        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    /// Initialize io_uring instance w/ `io_uring_setup` 
    /// syscall & passing default paramters struct. 
    ///     - kernel fills struct with actual vals and offsets.
    fn init_io_uring() ->  
        Result<(RawFd, iouring_abi::io_uring_params), io::Error> {
        let mut params = iouring_abi::io_uring_params::default();
        let fd =
            unsafe { 
                iouring_abi::syscall(
                    iouring_abi::SYS_IO_URING_SETUP, 
                    entries, 
                    &mut params)
            } 
            as RawFd;
    
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        result = Ok((fd, params));
    }

    /// Map the submission queue ring from the kernel
    /// using the fd/offsets obtained from `io_uring_setup`.
    fn get_sq_ring(
        params: &iouring_abi::io_uring_params, 
        fd: RawFd) 
        -> io::Result<*mut c_void> {
        let sq_ring_size = params.sq_off.array as usize + (
            params.sq_entries as usize * 4);

        mmap_ring(
            fd, 
            sq_ring_size, 
            iouring_abi::IORING_OFF_SQ_RING)
    }

    /// Map the completion queue ring from the kernel
    /// using the fd/offsets obtained from `io_uring_setup`.
    fn get_cq_ring(
        params: &iouring_abi::io_uring_params, 
        fd: RawFd) 
        -> io::Result<*mut c_void> {
        let cq_ring_size = params.cq_off.cqes as usize + (
            params.cq_entries as usize * 
            mem::size_of::<iouring_abi::io_uring_cqe>());

        mmap_ring(
            fd, 
            cq_ring_size, 
            iouring_abi::IORING_OFF_CQ_RING)
    }
}

impl SubmissionQueue {
    /// Available slots in SQ ring (tail - head)
    fn available(&self) -> u32 {
        unsafe {
            let head = (*self.head).load(Ordering::Acquire);
            let tail = (*self.tail).load(Ordering::Relaxed);
            self.ring_entries - tail.wrapping_sub(head)
        }
    }

    /// Prep a write operation in the next available SQE slot.
    fn prepare_write(
        &self, 
        file_fd: RawFd, 
        buffer: &[u8], 
        file_offset: u64, 
        user_data: u64) {
        unsafe {
            let tail = (*self.tail).load(Ordering::Relaxed);
            let index = tail & self.ring_mask;
            let sqe = &mut *self.sqes.add(index as usize);

            *sqe = iouring_abi::io_uring_sqe::default();
            sqe.opcode = iouring_abi::IORING_OP_WRITE;
            sqe.fd = file_fd;
            sqe.off = file_offset;
            sqe.addr = buffer.as_ptr() as u64;
            sqe.len = buffer.len() as u32;
            sqe.user_data = user_data;

            *self.array.add(index as usize) = index;
            /// Release publishes the SQE contents before the kernel 
            /// observes the new tail.
            (*self.tail).store(
                tail.wrapping_add(1), Ordering::Release);
        }
    }
}

impl CompletionQueue {
    /// Peek at the next completion in the CQ
    fn peek_one(&self) -> Option<(iouring_abi::io_uring_cqe, u32)> {
        let head = unsafe { (*self.head).load(Ordering::Relaxed) };
        let tail = unsafe { (*self.tail).load(Ordering::Acquire) };

        if head == tail {
            return None;
        }

        let index = (head & self.ring_mask) as usize;
        let cqe = unsafe { *self.cqes.add(index) };
        Some((cqe, head))
    }

    /// Advance the CQ head after processing completions.
    fn advance(&self, last_head: u32, count: u32) {
        let new_head = last_head.wrapping_add(count);
        unsafe {
            /// Release tells the kernel it may reuse CQ slots 
            /// after userspace read the CQE.
            (*self.head).store(new_head, Ordering::Release);
        }
    }
}

/// Helper to mmap a ring buffer from the kernel, given fd/size/offset.
fn mmap_ring(fd: RawFd, size: usize, offset: i64) -> io::Result<*mut c_void> {
    let mapping = unsafe {
        iouring_abi::mmap(
            ptr::null_mut(),
            size,
            iouring_abi::PROT_READ | iouring_abi::PROT_WRITE,
            iouring_abi::MAP_SHARED,
            fd,
            offset,
        )
    };

    if mapping == iouring_abi::MAP_FAILED {
        Err(io::Error::last_os_error())
    } else {
        Ok(mapping)
    }
}
