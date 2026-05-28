use std::ffi::c_void;

// x86_64 Linux syscall numbers for creating and driving an io_uring instance.
pub const SYS_IO_URING_SETUP: isize = 425;
pub const SYS_IO_URING_ENTER: isize = 426;

// Ask io_uring_enter to wait until at least the requested number of CQEs exist.
pub const IORING_ENTER_GETEVENTS: u32 = 1;

// Operation codes used by this example.
pub const IORING_OP_WRITE: u8 = 23;

// Special mmap offsets understood by the io_uring file descriptor.
pub const IORING_OFF_SQ_RING: i64 = 0;
pub const IORING_OFF_CQ_RING: i64 = 0x8000000;
pub const IORING_OFF_SQES: i64 = 0x10000000;

// mmap protection and sharing flags used for the kernel/user shared rings.
pub const PROT_READ: i32 = 1;
pub const PROT_WRITE: i32 = 2;
pub const MAP_SHARED: i32 = 1;

// mmap returns this sentinel pointer on failure instead of null.
pub const MAP_FAILED: *mut c_void = !0usize as *mut c_void;

// ABI binding to libc/kernel syscall entry points.
unsafe extern "C" {
    // Variadic syscall entry point. The arguments depend on the syscall number.
    pub fn syscall(num: isize, ...) -> isize;

    // Maps the io_uring rings and SQE array out of the ring file descriptor.
    pub fn mmap(
        addr: *mut c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
}

// Byte offsets for fields inside the mapped submission queue ring.
#[repr(C)]
#[derive(Default)]
pub struct io_sqring_offsets {
    // User-space reads this to see what the kernel has consumed.
    pub head: u32,
    // User-space updates this after placing SQEs into the submission array.
    pub tail: u32,
    // Ring index mask. Use `index & ring_mask` instead of `% ring_entries`.
    pub ring_mask: u32,
    // Number of slots in the submission ring.
    pub ring_entries: u32,
    // Kernel-owned submission queue flags.
    pub flags: u32,
    // Number of submissions dropped by the kernel.
    pub dropped: u32,
    // Offset of the u32 array that maps ring slots to SQE indexes.
    pub array: u32,
    // Reserved for ABI stability.
    pub resv1: u32,
    // Used by newer setup modes where userspace provides ring memory.
    pub user_addr: u64,
}

// Byte offsets for fields inside the mapped completion queue ring.
#[repr(C)]
#[derive(Default)]
pub struct io_cqring_offsets {
    // User-space advances this after it has consumed CQEs.
    pub head: u32,
    // Kernel advances this after it writes new CQEs.
    pub tail: u32,
    // Ring index mask. Use `index & ring_mask` instead of `% ring_entries`.
    pub ring_mask: u32,
    // Number of slots in the completion ring.
    pub ring_entries: u32,
    // Number of CQEs that could not fit in the completion ring.
    pub overflow: u32,
    // Offset of the first io_uring_cqe in the mapped CQ ring.
    pub cqes: u32,
    // Kernel-owned completion queue flags.
    pub flags: u32,
    // Reserved for ABI stability.
    pub resv1: u32,
    // Used by newer setup modes where userspace provides ring memory.
    pub user_addr: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct io_uring_params {
    // Params passed to io_uring_setup and filled in by the kernel.
    pub sq_entries: u32,           // Number of submission queue entries.
    pub cq_entries: u32,           // Number of completion queue entries.
    pub flags: u32,                // Setup flags requested by userspace.
    pub sq_thread_cpu: u32,        // CPU to run the SQ polling thread on, when enabled.
    pub sq_thread_idle: u32,       // Time in ms for the SQ polling thread to idle before sleeping.
    pub features: u32,             // Supported features returned by the kernel.
    pub wq_fd: u32,                // Workqueue fd when sharing worker state with another ring.
    pub resv: [u32; 3],            // Reserved padding so the kernel can extend the ABI.
    pub sq_off: io_sqring_offsets, // Offsets for SQ ring fields in shared memory refs
    pub cq_off: io_cqring_offsets, // Offsets for CQ ring fields in shared memory refs
}

// One submission queue entry: a single operation request for the kernel.
#[repr(C)]
#[derive(Default)]
pub struct io_uring_sqe {
    // Operation code, such as IORING_OP_NOP, READV, WRITEV, ACCEPT, etc.
    pub opcode: u8,
    // Per-SQE flags, such as fixed-file or linked-operation behavior.
    pub flags: u8,
    // I/O priority for operations that support it.
    pub ioprio: u16,
    // File descriptor the operation targets, when the opcode uses one.
    pub fd: i32,
    // File offset or opcode-specific 64-bit value.
    pub off: u64,
    // Buffer address, iovec address, sockaddr address, or opcode-specific pointer.
    pub addr: u64,
    // Buffer length, iovec count, or opcode-specific length.
    pub len: u32,
    // Operation-specific flags, historically named rw_flags in the kernel.
    pub cmd_flags: u32,
    // Opaque value copied back into the CQE so you can identify the request.
    pub user_data: u64,
    // Buffer group/index for registered or selected buffer operations.
    pub buf_index: u16,
    // Registered personality/credential id, when used.
    pub personality: u16,
    // Extra fd used by splice-like operations.
    pub splice_fd_in: i32,
    // Reserved/extension space; must keep the kernel ABI layout intact.
    pub pad2: [u64; 2],
}

// One completion queue entry: the kernel's result for a finished SQE.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct io_uring_cqe {
    pub user_data: u64, // The ID you passed in the submission entry.
    pub res: i32,       // Result: bytes/status on success, or a negative errno.
    pub flags: u32,     // Kernel flags, such as buffer selection metadata.
}
