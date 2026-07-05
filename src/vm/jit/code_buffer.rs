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

impl CodeBuffer {
    /// Allocate a new buffer with at least `initial_capacity` bytes.
    pub fn new(initial_capacity: usize) -> Self {
        let capacity = initial_capacity.max(4096);
        let buf = unsafe {
            libc::mmap(
                ptr::null_mut(),
                capacity,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                -1,
                0,
            )
        };
        if buf == libc::MAP_FAILED {
            panic!("JIT: mmap failed");
        }
        Self {
            buf: buf as *mut u8,
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
        let new_buf = unsafe {
            libc::mmap(
                ptr::null_mut(),
                new_cap,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
                -1,
                0,
            )
        };
        if new_buf == libc::MAP_FAILED {
            panic!("JIT: mmap grow failed");
        }
        unsafe {
            ptr::copy_nonoverlapping(self.buf, new_buf as *mut u8, self.len);
            libc::munmap(self.buf as *mut libc::c_void, self.capacity);
        }
        self.buf = new_buf as *mut u8;
        self.capacity = new_cap;
    }

    /// Finalize the buffer: make it executable and read-only (W^X).
    /// Returns the pointer to the start of executable code.
    pub fn finalize(&mut self) -> *const u8 {
        let page_size = 4096usize;
        let aligned_cap = (self.capacity + page_size - 1) & !(page_size - 1);
        let rc = unsafe {
            libc::mprotect(
                self.buf as *mut libc::c_void,
                aligned_cap,
                libc::PROT_READ | libc::PROT_EXEC,
            )
        };
        if rc != 0 {
            panic!("JIT: mprotect to RX failed");
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
                libc::munmap(self.buf as *mut libc::c_void, self.capacity);
            }
        }
    }
}
