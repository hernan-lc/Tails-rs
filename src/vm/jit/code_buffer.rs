use std::ptr;

/// A growable buffer backed by mmap'd memory that can be switched from
/// writable to executable.
pub struct CodeBuffer {
    buf: *mut u8,
    len: usize,
    capacity: usize,
}

// SAFETY: CodeBuffer is only accessed from the JIT compiler thread.
unsafe impl Send for CodeBuffer {}
unsafe impl Sync for CodeBuffer {}

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr;
    use windows_sys::Win32::System::Memory::{
        VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_EXECUTE_READ, PAGE_READWRITE,
    };

    pub unsafe fn alloc_executable(size: usize) -> *mut u8 {
        let ptr = VirtualAlloc(
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if ptr.is_null() {
            panic!("JIT: VirtualAlloc failed");
        }
        ptr as *mut u8
    }

    pub unsafe fn free(ptr: *mut u8, size: usize) {
        VirtualFree(ptr as *mut _, size, MEM_RELEASE);
    }

    pub unsafe fn make_executable(ptr: *mut u8, size: usize) {
        let mut old_protect = 0u32;
        let rc = VirtualProtect(ptr as *mut _, size, PAGE_EXECUTE_READ, &mut old_protect);
        if rc == 0 {
            panic!("JIT: VirtualProtect to RX failed");
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use libc::{
        mmap, mprotect, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ,
        PROT_WRITE,
    };
    use std::ptr;

    pub unsafe fn alloc_executable(size: usize) -> *mut u8 {
        let buf = mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_ANONYMOUS | MAP_PRIVATE,
            -1,
            0,
        );
        if buf == MAP_FAILED {
            panic!("JIT: mmap failed");
        }
        buf as *mut u8
    }

    pub unsafe fn free(ptr: *mut u8, size: usize) {
        munmap(ptr as *mut libc::c_void, size);
    }

    pub unsafe fn make_executable(ptr: *mut u8, size: usize) {
        let rc = mprotect(ptr as *mut libc::c_void, size, PROT_READ | PROT_EXEC);
        if rc != 0 {
            panic!("JIT: mprotect to RX failed");
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use libc::{
        mmap, mprotect, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_JIT, MAP_PRIVATE, PROT_EXEC,
        PROT_READ, PROT_WRITE,
    };
    use std::ptr;

    extern "C" {
        fn pthread_jit_write_protect_np(enable: libc::c_int);
    }

    pub unsafe fn alloc_executable(size: usize) -> *mut u8 {
        let buf = mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_ANONYMOUS | MAP_PRIVATE | MAP_JIT,
            -1,
            0,
        );
        if buf == MAP_FAILED {
            panic!("JIT: mmap failed on macOS");
        }
        buf as *mut u8
    }

    pub unsafe fn free(ptr: *mut u8, size: usize) {
        munmap(ptr as *mut libc::c_void, size);
    }

    pub unsafe fn make_executable(ptr: *mut u8, size: usize) {
        // Enable write access for JIT compilation
        pthread_jit_write_protect_np(0);
        let rc = mprotect(ptr as *mut libc::c_void, size, PROT_READ | PROT_EXEC);
        pthread_jit_write_protect_np(1);
        if rc != 0 {
            panic!("JIT: mprotect to RX failed on macOS");
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod platform {
    pub unsafe fn alloc_executable(size: usize) -> *mut u8 {
        panic!("JIT: unsupported platform");
    }
    pub unsafe fn free(_ptr: *mut u8, _size: usize) {}
    pub unsafe fn make_executable(_ptr: *mut u8, _size: usize) {
        panic!("JIT: unsupported platform");
    }
}

impl CodeBuffer {
    /// Allocate a new buffer with at least `initial_capacity` bytes.
    pub fn new(initial_capacity: usize) -> Self {
        let capacity = initial_capacity.max(4096);
        let buf = unsafe { platform::alloc_executable(capacity) };
        Self {
            buf,
            len: 0,
            capacity,
        }
    }

    /// Current write position (byte offset).
    pub fn offset(&self) -> usize {
        self.len
    }

    /// Emit a single byte.
    pub fn emit_byte(&mut self, b: u8) {
        self.grow(1);
        unsafe {
            *self.buf.add(self.len) = b;
        }
        self.len += 1;
    }

    /// Emit a 32-bit little-endian value.
    pub fn emit32(&mut self, val: u32) {
        self.grow(4);
        unsafe {
            ptr::write_unaligned(self.buf.add(self.len) as *mut u32, val);
        }
        self.len += 4;
    }

    /// Emit a 64-bit little-endian value.
    pub fn emit64(&mut self, val: u64) {
        self.grow(8);
        unsafe {
            ptr::write_unaligned(self.buf.add(self.len) as *mut u64, val);
        }
        self.len += 8;
    }

    /// Patch a 32-bit value at the given offset.
    pub fn patch32(&mut self, offset: usize, val: u32) {
        unsafe {
            ptr::write_unaligned(self.buf.add(offset) as *mut u32, val);
        }
    }

    /// Read a 32-bit value at the given offset.
    pub fn read32(&self, offset: usize) -> u32 {
        unsafe { ptr::read_unaligned(self.buf.add(offset) as *const u32) }
    }

    /// Ensure there is room for `additional` more bytes.
    fn grow(&mut self, additional: usize) {
        let needed = self.len + additional;
        if needed <= self.capacity {
            return;
        }
        let new_cap = (self.capacity * 2).max(needed).max(4096);
        let new_buf = unsafe { platform::alloc_executable(new_cap) };
        unsafe {
            ptr::copy_nonoverlapping(self.buf, new_buf, self.len);
            platform::free(self.buf, self.capacity);
        }
        self.buf = new_buf;
        self.capacity = new_cap;
    }

    /// Finalize the buffer: make it executable and read-only (W^X).
    /// Returns the pointer to the start of executable code.
    pub fn finalize(&mut self) -> *const u8 {
        let page_size = 4096usize;
        let aligned_cap = (self.capacity + page_size - 1) & !(page_size - 1);
        unsafe {
            platform::make_executable(self.buf, aligned_cap);
        }
        self.buf as *const u8
    }

    /// Returns the raw pointer (for use before finalize).
    pub fn as_ptr(&self) -> *const u8 {
        self.buf as *const u8
    }
}

impl Drop for CodeBuffer {
    fn drop(&mut self) {
        if !self.buf.is_null() && self.capacity > 0 {
            unsafe {
                platform::free(self.buf, self.capacity);
            }
        }
    }
}
