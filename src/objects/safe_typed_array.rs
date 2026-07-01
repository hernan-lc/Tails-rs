use super::js_array::{TypedArray, TypedArrayType};
use std::marker::PhantomData;

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
}

/// A type-safe reference to an element in a TypedArray
pub struct TypedArrayRef<'a, T> {
    ptr: *mut T,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> TypedArrayRef<'a, T> {
    /// Create a new TypedArrayRef from a raw pointer
    ///
    /// # Safety
    /// The pointer must be valid and properly aligned for type T
    pub unsafe fn new(ptr: *mut T) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the element
    ///
    /// # Safety
    /// The pointer must be valid
    pub unsafe fn as_ref(&self) -> &T {
        &*self.ptr
    }

    /// Get a mutable reference to the element
    ///
    /// # Safety
    /// The pointer must be valid and no other references exist
    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.ptr
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Get the raw mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

/// Safe typed array access functions
impl TypedArray {
    /// Get a safe reference to an element at the given index
    ///
    /// # Safety
    /// The index must be within bounds and the type T must match the element type
    pub unsafe fn get_ref<T>(&self, index: usize) -> Option<TypedArrayRef<'_, T>> {
        let element_size = Self::element_size(&self.kind);
        let byte_index = index * element_size;

        if byte_index + element_size > self.byte_length {
            return None;
        }

        let ptr = self.buffer.as_ptr().add(byte_index) as *mut T;
        Some(TypedArrayRef::new(ptr))
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
    fn test_typed_array_ref_creation() {
        let mut value = 42i32;
        let ptr = &mut value as *mut i32;

        unsafe {
            let mut ref_ = TypedArrayRef::new(ptr);
            assert_eq!(*ref_.as_ref(), 42);
            *ref_.as_mut() = 100;
            assert_eq!(value, 100);
        }
    }

    #[test]
    fn test_typed_array_get_ref() {
        let mut typed_array = TypedArray {
            kind: TypedArrayType::Int32Array,
            buffer: vec![0; 16],
            byte_length: 16,
            byte_offset: 0,
        };

        // Write some values
        typed_array.set_value(0, 10i32);
        typed_array.set_value(1, 20i32);
        typed_array.set_value(2, 30i32);
        typed_array.set_value(3, 40i32);

        unsafe {
            let ref0 = typed_array.get_ref::<i32>(0).unwrap();
            assert_eq!(*ref0.as_ref(), 10);

            let ref1 = typed_array.get_ref::<i32>(1).unwrap();
            assert_eq!(*ref1.as_ref(), 20);

            let ref2 = typed_array.get_ref::<i32>(2).unwrap();
            assert_eq!(*ref2.as_ref(), 30);

            let ref3 = typed_array.get_ref::<i32>(3).unwrap();
            assert_eq!(*ref3.as_ref(), 40);

            // Out of bounds should return None
            assert!(typed_array.get_ref::<i32>(4).is_none());
        }
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
