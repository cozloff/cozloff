### io_uring instance:

<b>std::os::fd::RawFd</b>
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

--- 

### Full Chain:
When you use a RawFd in your io_uring code, the kernel follows:

1. Kernel looks at <b>task_struct</b> of running program.
2. Goes into <b>files_struct</b> with RawFd as index into <b>fd_array</b>
3. Finds the <b>struct file</b> at that index and performs I/O.