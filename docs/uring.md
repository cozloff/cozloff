## io_uring instance:

##### <b>std::os::fd::RawFd</b>
<u>File Descriptors (fd)</u>: Unix mechanism for interacting with I/O via tracking user file position / mode.
<u>RawFd</u>: Integer index into private table maintained by the Linux kernel for a specific process, called the File Descriptor Table. 
- i. <b>Rust</b>: Holds a RawFd (e.g. int #3) 
- ii. <b>Process Table</b>: Kernel looks at "Slot #3" for your process, specifically on the "File Descriptor Table". 
- iii. <b>File Description</b>: Slot #3 contains a pointer to a "File Description" - tracks user's current position in the file and whether in read or write mode. 
- iv. <b>Inode</b>: Description points to the actual data on the disk. 

1. <span style="color:#D6B4FC; font-weight:bold;">task_struct</span>
In the Linux kernel, every process (and thread) is represented by a task_struct in kernel code execution. Containing it's name, priority, memory map, and its <b>...files</b>

2. <span style="color:#D6B4FC; font-weight:bold;">files_struct</span>
Inside the <span style="color:#D6B4FC">task_struct</span>, there is a pointer to a structure called <span style="color:#D6B4FC">files_struct</span>.
    - Manages all open files for that process.
    - Multiple threads can point to same struct sharing fd.

3. <span style="color:#D6B4FC; font-weight:bold;">fd_array</span>
Inside that <span style="color:#D6B4FC">files_struct</span> is an array of pointers called the <b>File Descriptor Table.</b>
    - This is where <b>RawFd</b> lives.
    - e.g. If RawFd = 3 : fd_array[3]

4. <span style="color:#D6B4FC; font-weight:bold;">struct file</span>
The pointer of fd_array[x] points to a struct file. This is a kernel object that represents an open file. 
    - <b>inode</b>- what file is it on disk?
    - <b>f_pos</b>- where are you currently reading? (offset)
    - <b>f_mode</b>- do you have permission to write? (flags) 

##### Full Chain:
When you use a RawFd in your io_uring code, the kernel follows:

1. Kernel looks at <b>task_struct</b> of running program.
2. Goes into <b>files_struct</b> with RawFd as index into <b>fd_array</b>
3. Finds the <b>struct file</b> at that index and performs I/O.

---
##### Submission Queue, Completion Queue:
<b>Linux CPU Scheduler</b>: <span style="color:#D6B4FC; font-weight:bold;">EEVDF</span> (Earliest Eligible Virtual Deadline First)
- Virtual Runtime (<span style="color:#D6B4FC; font-weight:bold;">vruntime</span>): instead of rigid time slices, Linux tracks how much CPU time each thread consumed. 
    - <span style="color:#D6B4FC; font-weight:bold;">Runqueue</span>: Every CPU core maintains a red-black tree (self-balancing binary search tree) of tasks that are ready to run, sorted by vruntime.
    - Selection: Scheduler picks the task with lowest vruntime.
    - EEVDF <span style="color:#D6B4FC; font-weight:bold;">Deadline</span>: While CFS (Completely Fair Scheduler) just chose the most starved task, EEVDF uses a deadline calculated from task's priority (nice value). If a task requests a small time chunk, it gets an earlier deadline, making it responsive for lag-sensitive interactive apps. 
    - Yielding CPU: A thread stops when its time <span style="color:#D6B4FC; font-weight:bold;">quantum</span> expires, it's preempted by a higher-priority task, or it explicitly blocks (goes to sleep) waiting for external data.

<b>SQ/CQ Framework</b> (io_uring, NVMe drivers, etc)
Goal: Minimize scheduler interference. 

- Interrupt-Driven Mode (classical): if app submits a read request to SQ and goes to sleep waiting for CQ to fill:
    - <u>Blocking (sleep)</u>: app thread checks CQ, empty. Thread calls func to wait, <span style="color:#D6B4FC; font-weight:bold;">TASK_RUNNING</span> -> <span style="color:#D6B4FC; font-weight:bold;">TASK_INTERRUPTIBLE</span>. Scheduler immediately removes the thread from CPU runqueue. 
    - <u>Hardware interrupt</u>: when data is fetched, hardware sends electrical signal-Hardware Interrupt(IRQ)-to the CPU.
    - <u>Kernel Step</u>: CPU pauses to run kernel's Interrupt Service Routine (ISR). The kernel takes the data from the hardware, formats it into a CQE, and pushes it onto CQ in memory. 
    - <u>Waking Thread</u>: Kernel changes app thread state back to <span style="color:#D6B4FC; font-weight:bold;">TASK_RUNNING</span> and back into runqueue. vruntime will be low, so the scheduler will likely context-switch it back onto a CPU core very quickly to consume the CQE. 

- Kernel Polling Mode (fast):
Hardware interrupts and scheduler context switches take time. Linux allows you to bypass the scheduler's blocking mechanics entirely using Kernel Polling (IORING_SETUP_SQPOLL).
    - <u>Dedicated Kernel Thread</u>: When you init the queues, kernel spawns a dedicated background thread (io_uring-sq).
    - <u>No Interrupts, Just Looping</u>: Kernel thread is bound to a specific CPU core and sits in a tight ```while(true)```,constantly staring at the SQ. 
    - <u>Zero Context Switching</u>: When your app pushes item to the SQ, there's no interrupt trigger, the kernel thread simply spots the entry on its next loop and instantly hands it to the hardware driver, and checks hardware registers for completions. 
    - <u>App never sleeps</u>: app thread can poll the CQ in user-space. Neither thread ever goes to sleep, meaning the Linux scheduler is bypassed for I/O operations. The threads just pinned to CPU cores, crunching data at maximum hardware limits. 


<b>Two Hidden Engines</b>: 
- Hardware Subsystem: NVMe, NIC, etc execute processes with their own onboard controllers and memory. 
    - DMA: kernel takes SQ, translates vmem addr -> physmem addr, and passes to the NVMe SSD controller.
    - NVMe drive's onboard processors independently pulls or pushes data over PCIe into RAM via DMA. 

- Kernel io_worker safety net: 
For ops that can't be offloaded to hardware, there's a specialized pool of internal kernel threads called io_wq (io work queues).
    - system proc list as io_w-xxx
    - belong to kernel -> managed by scheduler, live in kernel space. 
    - pick up SQE on behalf of app, drop results to CQ. 