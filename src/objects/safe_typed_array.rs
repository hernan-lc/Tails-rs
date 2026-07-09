use super::js_array::{NeBytes, TypedArray, TypedArrayType};

/// A safe wrapper around TypedArray operations
pub struct SafeTypedArray<'a> {
    inner: &'a mut TypedArray,
}

impl<'a> SafeTypedArray<'a> {
    /// Create a new SafeTypedArray from a mutable reference
    pub fn new(inner: &'a mut TypedArray) -> Self {
        Self { inner }
    }

    /// Get the kind of the typed array
    pub fn kind(&self) -> &TypedArrayType {
        &self.inner.kind
    }

    /// Get the byte length of the typed array
    pub fn byte_length(&self) -> usize {
        self.inner.byte_length
    }

    /// Get the byte offset of the typed array
    pub fn byte_offset(&self) -> usize {
        self.inner.byte_offset
    }

    /// Get the length (number of elements) of the typed array
    pub fn length(&self) -> usize {
        let element_size = Self::element_size(&self.inner.kind);
        self.inner
            .byte_length
            .checked_div(element_size)
            .unwrap_or(0)
    }

    /// Get the element size for a given kind
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

    /// Get a reference to the underlying TypedArray
    pub fn inner(&self) -> &TypedArray {
        self.inner
    }

    /// Get a mutable reference to the underlying TypedArray
    pub fn inner_mut(&mut self) -> &mut TypedArray {
        self.inner
    }

    /// Safe element read via `NeBytes`.
    pub fn get<T: NeBytes>(&self, index: usize) -> Option<T> {
        self.inner.get(index)
    }

    /// Safe element write via `NeBytes`.
    pub fn set_value<T: NeBytes>(&mut self, index: usize, value: T) {
        self.inner.set_value(index, value);
    }
}

/// Safe typed array access helpers on `TypedArray` itself.
impl TypedArray {
    /// Get an element by value (fully safe, no raw pointers).
    pub fn get_element<T: NeBytes>(&self, index: usize) -> Option<T> {
        self.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_typed_array_length() {
        let mut typed_array = TypedArray {
            kind: TypedArrayType::Int32Array,
            buffer: vec![0; 16],
            byte_length: 16,
            byte_offset: 0,
        };

        let safe_array = SafeTypedArray::new(&mut typed_array);
        assert_eq!(safe_array.length(), 4);
        assert_eq!(safe_array.byte_length(), 16);
    }

    #[test]
    fn test_safe_typed_array_kind() {
        let mut typed_array = TypedArray {
            kind: TypedArrayType::Float64Array,
            buffer: vec![0; 32],
            byte_length: 32,
            byte_offset: 0,
        };

        let safe_array = SafeTypedArray::new(&mut typed_array);
        assert!(matches!(safe_array.kind(), TypedArrayType::Float64Array));
    }

    #[test]
    fn test_safe_typed_array_byte_offset() {
        let mut typed_array = TypedArray {
            kind: TypedArrayType::Uint8Array,
            buffer: vec![0; 10],
            byte_length: 10,
            byte_offset: 5,
        };

        let safe_array = SafeTypedArray::new(&mut typed_array);
        assert_eq!(safe_array.byte_offset(), 5);
    }

    #[test]
    fn test_typed_array_safe_get_set() {
        let mut typed_array = TypedArray {
            kind: TypedArrayType::Int32Array,
            buffer: vec![0; 16],
            byte_length: 16,
            byte_offset: 0,
        };

        typed_array.set_value(0, 10i32);
        typed_array.set_value(1, 20i32);
        typed_array.set_value(2, 30i32);
        typed_array.set_value(3, 40i32);

        assert_eq!(typed_array.get::<i32>(0), Some(10));
        assert_eq!(typed_array.get::<i32>(1), Some(20));
        assert_eq!(typed_array.get::<i32>(2), Some(30));
        assert_eq!(typed_array.get::<i32>(3), Some(40));
        assert!(typed_array.get::<i32>(4).is_none());
    }

    #[test]
    fn test_element_size() {
        assert_eq!(SafeTypedArray::element_size(&TypedArrayType::Int8Array), 1);
        assert_eq!(SafeTypedArray::element_size(&TypedArrayType::Uint8Array), 1);
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::Uint8ClampedArray),
            1
        );
        assert_eq!(SafeTypedArray::element_size(&TypedArrayType::Int16Array), 2);
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::Uint16Array),
            2
        );
        assert_eq!(SafeTypedArray::element_size(&TypedArrayType::Int32Array), 4);
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::Uint32Array),
            4
        );
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::Float32Array),
            4
        );
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::Float64Array),
            8
        );
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::BigInt64Array),
            8
        );
        assert_eq!(
            SafeTypedArray::element_size(&TypedArrayType::BigUint64Array),
            8
        );
    }
}
