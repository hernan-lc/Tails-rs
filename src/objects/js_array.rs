use crate::objects::Value;

/// Types that can be stored in a TypedArray via native-endian byte conversion.
/// Implemented for all standard TypedArray element types without `unsafe`.
pub trait NeBytes: Copy {
    const SIZE: usize;
    fn read_from(bytes: &[u8]) -> Self;
    fn write_to(self, bytes: &mut [u8]);
}

macro_rules! impl_ne_bytes {
    ($($t:ty),+ $(,)?) => {$(
        impl NeBytes for $t {
            const SIZE: usize = std::mem::size_of::<$t>();
            fn read_from(bytes: &[u8]) -> Self {
                let mut arr = [0u8; std::mem::size_of::<$t>()];
                arr.copy_from_slice(&bytes[..std::mem::size_of::<$t>()]);
                <$t>::from_ne_bytes(arr)
            }
            fn write_to(self, bytes: &mut [u8]) {
                bytes[..std::mem::size_of::<$t>()].copy_from_slice(&self.to_ne_bytes());
            }
        }
    )+};
}

impl_ne_bytes!(u8, i8, u16, i16, u32, i32, u64, i64);

impl NeBytes for f32 {
    const SIZE: usize = 4;
    fn read_from(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&bytes[..4]);
        f32::from_ne_bytes(arr)
    }
    fn write_to(self, bytes: &mut [u8]) {
        bytes[..4].copy_from_slice(&self.to_ne_bytes());
    }
}

impl NeBytes for f64 {
    const SIZE: usize = 8;
    fn read_from(bytes: &[u8]) -> Self {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        f64::from_ne_bytes(arr)
    }
    fn write_to(self, bytes: &mut [u8]) {
        bytes[..8].copy_from_slice(&self.to_ne_bytes());
    }
}

#[derive(Debug, Clone)]
pub enum TypedArrayType {
    Int8Array,
    Uint8Array,
    Uint8ClampedArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Float32Array,
    Float64Array,
    BigInt64Array,
    BigUint64Array,
}

#[derive(Debug, Clone)]
pub struct JsArray {
    pub elements: Vec<Value>,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct TypedArray {
    pub kind: TypedArrayType,
    pub buffer: Vec<u8>,
    pub byte_length: usize,
    pub byte_offset: usize,
}

impl JsArray {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            length: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
            length: 0,
        }
    }

    pub fn push(&mut self, value: Value) {
        self.elements.push(value);
        self.length = self.elements.len();
    }

    pub fn pop(&mut self) -> Option<Value> {
        let value = self.elements.pop();
        self.length = self.elements.len();
        value
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.elements.get(index)
    }

    pub fn set(&mut self, index: usize, value: Value) {
        if index >= self.elements.len() {
            self.elements.resize_with(index + 1, || Value::Undefined);
        }
        self.elements[index] = value;
        self.length = self.elements.len();
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn slice(&self, start: usize, end: usize) -> JsArray {
        let end = end.min(self.elements.len());
        let start = start.min(end);

        JsArray {
            elements: self.elements[start..end].to_vec(),
            length: end - start,
        }
    }

    pub fn splice(&mut self, start: usize, delete_count: usize, items: Vec<Value>) -> Vec<Value> {
        let start = start.min(self.elements.len());
        let delete_count = delete_count.min(self.elements.len() - start);

        let removed: Vec<Value> = self.elements.drain(start..start + delete_count).collect();

        for (i, item) in items.into_iter().enumerate() {
            self.elements.insert(start + i, item);
        }

        self.length = self.elements.len();
        removed
    }
}

impl TypedArray {
    pub fn new(kind: TypedArrayType, length: usize) -> Self {
        let element_size = Self::element_size(&kind);
        let byte_length = length * element_size;

        Self {
            kind,
            buffer: vec![0; byte_length],
            byte_length,
            byte_offset: 0,
        }
    }

    pub fn from_buffer(kind: TypedArrayType, buffer: Vec<u8>, byte_offset: usize) -> Self {
        let byte_length = buffer.len();

        Self {
            kind,
            buffer,
            byte_length,
            byte_offset,
        }
    }

    /// Read an element at `index` using native-endian byte conversion.
    /// Fully safe: bounds-checked slice copy, no raw pointers.
    pub fn get<T: NeBytes>(&self, index: usize) -> Option<T> {
        let element_size = T::SIZE;
        let byte_index = index.checked_mul(element_size)?;
        let end = byte_index.checked_add(element_size)?;
        if end > self.byte_length || end > self.buffer.len() {
            return None;
        }
        Some(T::read_from(&self.buffer[byte_index..end]))
    }

    /// Write an element at `index` using native-endian byte conversion.
    /// Fully safe: grows the buffer if needed, then copies bytes.
    pub fn set_value<T: NeBytes>(&mut self, index: usize, value: T) {
        let element_size = T::SIZE;
        let byte_index = index.saturating_mul(element_size);
        let end = byte_index + element_size;

        if end > self.byte_length {
            self.buffer.resize(end, 0);
            self.byte_length = self.buffer.len();
        }

        value.write_to(&mut self.buffer[byte_index..end]);
    }

    pub fn fill(&mut self, value: i32) {
        let element_size = Self::element_size(&self.kind);
        let count = self.byte_length / element_size;

        for i in 0..count {
            match &self.kind {
                TypedArrayType::Int8Array
                | TypedArrayType::Uint8Array
                | TypedArrayType::Uint8ClampedArray => {
                    self.set_value(i, value as u8);
                }
                TypedArrayType::Int16Array | TypedArrayType::Uint16Array => {
                    self.set_value(i, value as u16);
                }
                TypedArrayType::Int32Array | TypedArrayType::Uint32Array => {
                    self.set_value(i, value as u32);
                }
                TypedArrayType::Float32Array => {
                    self.set_value(i, value as f32);
                }
                TypedArrayType::Float64Array => {
                    self.set_value(i, value as f64);
                }
                _ => {}
            }
        }
    }

    pub fn element_size(kind: &TypedArrayType) -> usize {
        match kind {
            TypedArrayType::Int8Array
            | TypedArrayType::Uint8Array
            | TypedArrayType::Uint8ClampedArray => 1,
            TypedArrayType::Int16Array | TypedArrayType::Uint16Array => 2,
            TypedArrayType::Int32Array
            | TypedArrayType::Uint32Array
            | TypedArrayType::Float32Array => 4,
            TypedArrayType::Float64Array
            | TypedArrayType::BigInt64Array
            | TypedArrayType::BigUint64Array => 8,
        }
    }
}

impl Default for JsArray {
    fn default() -> Self {
        Self::new()
    }
}
