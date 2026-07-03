use std::io::Write;
use std::path::Path;

use tails_native_macros::{tails_function, tails_module};

// ============================================================================
// Public API for direct Rust usage
// ============================================================================

pub fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn write_file(path: &str, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn mkdir(path: &str, recursive: bool) -> std::io::Result<()> {
    if recursive {
        std::fs::create_dir_all(path)
    } else {
        std::fs::create_dir(path)
    }
}

pub fn readdir(path: &str) -> std::io::Result<Vec<String>> {
    let entries: Vec<String> = std::fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    Ok(entries)
}

#[derive(Debug, Clone)]
pub struct FileStat {
    pub size: u64,
    pub is_file: bool,
    pub is_directory: bool,
    pub is_symbolic_link: bool,
    pub mode: Option<u32>,
    pub mtime_ms: Option<f64>,
    pub birthtime_ms: Option<f64>,
}

pub fn stat(path: &str) -> std::io::Result<FileStat> {
    let metadata = std::fs::metadata(path)?;

    let mode = {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Some(metadata.mode())
        }
        #[cfg(not(unix))]
        {
            None
        }
    };

    let mtime_ms = metadata.modified().ok().and_then(|t| {
        t.duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs_f64() * 1000.0)
    });

    let birthtime_ms = {
        #[cfg(unix)]
        {
            metadata.created().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs_f64() * 1000.0)
            })
        }
        #[cfg(not(unix))]
        {
            None
        }
    };

    Ok(FileStat {
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_directory: metadata.is_dir(),
        is_symbolic_link: metadata.file_type().is_symlink(),
        mode,
        mtime_ms,
        birthtime_ms,
    })
}

pub fn unlink(path: &str) -> std::io::Result<()> {
    std::fs::remove_file(path)
}

pub fn rm(path: &str, recursive: bool) -> std::io::Result<()> {
    if recursive {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

pub fn copy_file(src: &str, dest: &str) -> std::io::Result<u64> {
    std::fs::copy(src, dest)
}

pub fn rename(old_path: &str, new_path: &str) -> std::io::Result<()> {
    std::fs::rename(old_path, new_path)
}

pub fn append_file(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(content.as_bytes())
}

// ============================================================================
// Base64 helpers (for binary I/O)
// ============================================================================

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(data: &str) -> Option<Vec<u8>> {
    const TABLE: [i8; 256] = {
        let mut table = [(-1i8); 256];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < chars.len() {
            table[chars[i] as usize] = i as i8;
            i += 1;
        }
        table[b'=' as usize] = 0;
        table
    };

    let data = data.trim_end_matches('=');
    if data.is_empty() {
        return Some(Vec::new());
    }

    let mut output = Vec::with_capacity(data.len() * 3 / 4);
    for chunk in data.as_bytes().chunks(4) {
        let mut buf = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            let val = TABLE[b as usize];
            if val < 0 {
                return None;
            }
            buf[i] = val as u8;
        }
        let triple = ((buf[0] as u32) << 18)
            | ((buf[1] as u32) << 12)
            | ((buf[2] as u32) << 6)
            | (buf[3] as u32);
        output.push((triple >> 16) as u8);
        if chunk.len() > 2 {
            output.push((triple >> 8) as u8);
        }
        if chunk.len() > 3 {
            output.push(triple as u8);
        }
    }
    Some(output)
}

// ============================================================================
// Native module (cdylib FFI exports)
// ============================================================================

/// A buffered streaming reader, analogous to Node's
/// `fs.createReadStream(path)`. The handle is owned by the
/// `fs_native::STREAMS` table and is removed when the user calls
/// `stream_close` (or drops the `id` on the JS side).
pub struct ReadStream {
    file: std::fs::File,
    size: u64,
}

impl ReadStream {
    pub fn open(path: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        Ok(Self { file, size })
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    /// Read up to `chunk_size` bytes. Returns `Ok(None)` at EOF.
    /// Sets `*hit_eof` to `true` when the read returned fewer bytes
    /// than requested (a short read at end-of-file); callers use that
    /// to surface `{done: true}` to the JS caller even though some
    /// bytes were returned in the same call.
    pub fn read_chunk(
        &mut self,
        chunk_size: usize,
        hit_eof: &mut bool,
    ) -> std::io::Result<Option<Vec<u8>>> {
        use std::io::Read;
        let mut buf = vec![0u8; chunk_size];
        match self.file.read(&mut buf)? {
            0 => Ok(None),
            n => {
                buf.truncate(n);
                // A short read (< chunk_size) means we've hit EOF in
                // the middle of the chunk. The next `read()` will
                // return 0, but the *current* call still produced
                // data, so the JS caller expects `{ok: true, done:
                // true, data: "..."}` rather than `{ok: true, done:
                // false, data: "..."}` followed by an empty chunk.
                if n < chunk_size {
                    *hit_eof = true;
                }
                Ok(Some(buf))
            }
        }
    }
}

/// A polling file-system watcher. The `notify` crate is too heavy
/// for the cdylib surface, so we use a simpler design: every
/// `interval_ms` we walk the watched path (file or directory) and
/// diff the entries against the previous snapshot. New entries
/// become `create` events, removed ones become `delete` events,
/// and size/mtime changes become `modify` events.
pub struct FileWatcher {
    path: std::path::PathBuf,
    interval: std::time::Duration,
    /// Last observed `(path, size, mtime_ms)` snapshot. `None` means
    /// we have not yet produced the initial snapshot.
    snapshot: Option<std::collections::HashMap<String, (u64, Option<u64>)>>,
    /// Pending events to drain on the next `poll()`.
    events: Vec<serde_json::Value>,
    /// Last time we walked the watched path.
    last_walk: std::time::Instant,
}

impl FileWatcher {
    pub fn new(path: &str, interval_ms: u64) -> std::io::Result<Self> {
        let p = std::path::PathBuf::from(path);
        if !p.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("watch path does not exist: {}", path),
            ));
        }
        // Take an initial snapshot immediately so the FIRST `poll()`
        // call after the watcher is set up has something to diff
        // against. Without this, the first `poll()` would lazy-init
        // the snapshot after the caller had already mutated the
        // watched directory (e.g. created a new file), and the
        // mutation would never appear as a "create" event.
        let interval_dur = std::time::Duration::from_millis(interval_ms);
        // Initialise last_walk *before* the current time by the interval
        // duration so the very first `poll()` call always triggers a re-walk.
        // Without this, changes made between `watch()` and the first `poll()`
        // would silently miss the snapshot diff window.
        let mut me = Self {
            path: p,
            interval: interval_dur,
            snapshot: None,
            events: Vec::new(),
            last_walk: std::time::Instant::now() - interval_dur,
        };
        me.snapshot = Some(me.walk());
        Ok(me)
    }

    fn walk(&self) -> std::collections::HashMap<String, (u64, Option<u64>)> {
        let mut out = std::collections::HashMap::new();
        let metadata = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return out,
        };
        if metadata.is_file() {
            let key = self.path.to_string_lossy().to_string();
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64);
            out.insert(key, (metadata.len(), mtime));
        } else if metadata.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&self.path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Ok(md) = entry.metadata() {
                        let key = p.to_string_lossy().to_string();
                        let mtime = md
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_millis() as u64);
                        out.insert(key, (md.len(), mtime));
                    }
                }
            }
        }
        out
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Drain pending events, walking the watched path if the poll
    /// interval has elapsed.
    pub fn poll(&mut self) -> Vec<serde_json::Value> {
        if self.last_walk.elapsed() >= self.interval || self.snapshot.is_none() {
            let current = self.walk();
            let now = Self::now_ms();
            if let Some(prev) = self.snapshot.take() {
                for (k, v) in &current {
                    match prev.get(k) {
                        None => self.events.push(serde_json::json!({
                            "type": "create",
                            "path": k,
                            "size": v.0,
                            "timeMs": now,
                        })),
                        Some((old_size, old_mtime)) => {
                            if old_size != &v.0 || old_mtime != &v.1 {
                                self.events.push(serde_json::json!({
                                    "type": "modify",
                                    "path": k,
                                    "size": v.0,
                                    "timeMs": now,
                                }));
                            }
                        }
                    }
                }
                for (k, v) in &prev {
                    if !current.contains_key(k) {
                        self.events.push(serde_json::json!({
                            "type": "delete",
                            "path": k,
                            "size": v.0,
                            "timeMs": now,
                        }));
                    }
                }
            }
            self.snapshot = Some(current);
            self.last_walk = std::time::Instant::now();
        }
        std::mem::take(&mut self.events)
    }
}

#[tails_module(name = "tails-fs")]
mod fs_native {
    use super::*;
    #[tails_function]
    pub fn read_file(path: String) -> String {
        super::read_file(&path).unwrap_or_default()
    }

    #[tails_function]
    pub fn write_file(path: String, content: String) -> bool {
        super::write_file(&path, &content).is_ok()
    }

    #[tails_function]
    pub fn exists(path: String) -> bool {
        super::exists(&path)
    }

    #[tails_function]
    pub fn mkdir(path: String, recursive: bool) -> bool {
        super::mkdir(&path, recursive).is_ok()
    }

    #[tails_function]
    pub fn readdir(path: String) -> String {
        super::readdir(&path)
            .map(|entries| serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|_| "[]".to_string())
    }

    #[tails_function]
    pub fn stat(path: String) -> String {
        match super::stat(&path) {
            Ok(s) => serde_json::json!({
                "size": s.size,
                "isFile": s.is_file,
                "isDirectory": s.is_directory,
                "isSymbolicLink": s.is_symbolic_link,
                "mode": s.mode,
                "mtimeMs": s.mtime_ms,
                "birthtimeMs": s.birthtime_ms,
            })
            .to_string(),
            Err(_) => "{}".to_string(),
        }
    }

    #[tails_function]
    pub fn unlink(path: String) -> bool {
        super::unlink(&path).is_ok()
    }

    #[tails_function]
    pub fn rm(path: String, recursive: bool) -> bool {
        super::rm(&path, recursive).is_ok()
    }

    #[tails_function]
    pub fn copy_file(src: String, dest: String) -> f64 {
        super::copy_file(&src, &dest).unwrap_or(0) as f64
    }

    #[tails_function]
    pub fn rename(old_path: String, new_path: String) -> bool {
        super::rename(&old_path, &new_path).is_ok()
    }

    #[tails_function]
    pub fn append_file(path: String, content: String) -> bool {
        super::append_file(&path, &content).is_ok()
    }

    #[tails_function]
    pub fn read_file_bytes(path: String) -> String {
        match std::fs::read(&path) {
            Ok(bytes) => {
                let encoded = base64_encode(&bytes);
                serde_json::to_string(&encoded).unwrap_or_default()
            }
            Err(_) => "".to_string(),
        }
    }

    #[tails_function]
    pub fn write_file_bytes(path: String, data: String) -> bool {
        match base64_decode(&data) {
            Some(bytes) => std::fs::write(&path, &bytes).is_ok(),
            None => false,
        }
    }

    #[tails_function]
    pub fn chmod(path: String, mode: f64) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode as u32)).is_ok()
        }
        #[cfg(not(unix))]
        {
            let _ = (path, mode);
            false
        }
    }

    #[tails_function]
    pub fn lstat(path: String) -> String {
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                #[cfg(unix)]
                let mode = {
                    use std::os::unix::fs::MetadataExt;
                    Some(metadata.mode())
                };
                #[cfg(not(unix))]
                let mode: Option<u32> = None;

                let mtime_ms = metadata.modified().ok().and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs_f64() * 1000.0)
                });

                serde_json::json!({
                    "size": metadata.len(),
                    "isFile": metadata.is_file(),
                    "isDirectory": metadata.is_dir(),
                    "isSymbolicLink": metadata.file_type().is_symlink(),
                    "mode": mode,
                    "mtimeMs": mtime_ms,
                })
                .to_string()
            }
            Err(_) => "{}".to_string(),
        }
    }

    #[tails_function]
    pub fn readlink(path: String) -> String {
        std::fs::read_link(&path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    #[tails_function]
    pub fn symlink(src: String, dest: String) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&src, &dest).is_ok()
        }
        #[cfg(not(unix))]
        {
            let _ = (src, dest);
            false
        }
    }

    #[tails_function]
    pub fn realpath(path: String) -> String {
        std::fs::canonicalize(&path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path)
    }

    // -----------------------------------------------------------------
    // Streams & Watchers (v0.5.0 roadmap additions)
    // -----------------------------------------------------------------

    /// Open a file for streaming reads. The handle id can be passed
    /// to `stream_read`, `stream_close`, etc. Returns a JSON object
    /// describing the stream (path, id, size).
    #[tails_function]
    pub fn create_read_stream(path: String) -> String {
        let id = __tails_next_id();
        let stream = super::ReadStream::open(&path);
        match stream {
            Ok(stream) => {
                let size = stream.size();
                STREAMS.lock().unwrap().insert(id, stream);
                serde_json::json!({
                    "ok": true,
                    "id": id,
                    "path": path,
                    "size": size,
                })
                .to_string()
            }
            Err(e) => serde_json::json!({
                "ok": false,
                "error": format!("ENOENT: cannot open '{}': {}", path, e),
            })
            .to_string(),
        }
    }

    /// Read the next chunk of bytes (base64-encoded) from a stream
    /// opened with `create_read_stream`. Returns `{done: true}` when
    /// the stream is exhausted.
    #[tails_function]
    pub fn stream_read(id: f64, chunk_size: f64) -> String {
        let id = id as u32;
        let chunk_size = chunk_size.max(1.0) as usize;
        let mut guard = STREAMS.lock().unwrap();
        match guard.get_mut(&id) {
            Some(stream) => {
                let mut hit_eof = false;
                match stream.read_chunk(chunk_size, &mut hit_eof) {
                    Ok(Some(bytes)) => serde_json::json!({
                        "ok": true,
                        "done": hit_eof,
                        "data": super::base64_encode(&bytes),
                    })
                    .to_string(),
                    Ok(None) => serde_json::json!({
                        "ok": true,
                        "done": true,
                    })
                    .to_string(),
                    Err(e) => serde_json::json!({
                        "ok": false,
                        "error": format!("read error: {}", e),
                    })
                    .to_string(),
                }
            }
            None => serde_json::json!({
                "ok": false,
                "error": "invalid stream id",
            })
            .to_string(),
        }
    }

    /// Close a stream opened with `create_read_stream`.
    #[tails_function]
    pub fn stream_close(id: f64) -> bool {
        STREAMS.lock().unwrap().remove(&(id as u32)).is_some()
    }

    /// Start a watcher on a path. Returns a JSON object with the
    /// watcher id. The watcher polls every `interval_ms` (default
    /// 100ms) and accumulates events in a queue readable via
    /// `watch_poll`. Use `watch_close` to stop.
    #[tails_function]
    pub fn watch(path: String, interval_ms: f64) -> String {
        let id = __tails_next_id();
        let interval = if interval_ms < 10.0 { 100 } else { interval_ms as u64 };
        let watcher = super::FileWatcher::new(&path, interval);
        match watcher {
            Ok(watcher) => {
                WATCHERS.lock().unwrap().insert(id, watcher);
                serde_json::json!({
                    "ok": true,
                    "id": id,
                    "path": path,
                    "intervalMs": interval,
                })
                .to_string()
            }
            Err(e) => serde_json::json!({
                "ok": false,
                "error": format!("cannot watch '{}': {}", path, e),
            })
            .to_string(),
        }
    }

    /// Drain pending events from a watcher. Returns a JSON array
    /// of `{type, path, timeMs}` events. Empty array if no events.
    #[tails_function]
    pub fn watch_poll(id: f64) -> String {
        let id = id as u32;
        let mut guard = WATCHERS.lock().unwrap();
        match guard.get_mut(&id) {
            Some(watcher) => {
                let events = watcher.poll();
                serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())
            }
            None => "[]".to_string(),
        }
    }

    /// Stop a watcher.
    #[tails_function]
    pub fn watch_close(id: f64) -> bool {
        WATCHERS.lock().unwrap().remove(&(id as u32)).is_some()
    }

    // Per-instance state tables. Using a function-local `__tails_`
    // helper for the id counter so the `#[tails_module]` macro
    // does not export it as a JS function.
    fn __tails_next_id() -> u32 {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    static STREAMS: once_cell::sync::Lazy<
        std::sync::Mutex<std::collections::HashMap<u32, super::ReadStream>>,
    > = once_cell::sync::Lazy::new(|| {
        std::sync::Mutex::new(std::collections::HashMap::new())
    });
    static WATCHERS: once_cell::sync::Lazy<
        std::sync::Mutex<std::collections::HashMap<u32, super::FileWatcher>>,
    > = once_cell::sync::Lazy::new(|| {
        std::sync::Mutex::new(std::collections::HashMap::new())
    });
}
