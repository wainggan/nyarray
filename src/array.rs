//! stack-allocated array structure.
//! similar to `Vec` in functionality, except [`Array`] lives on the 'stack',
//! lending it well for scratch arrays.
//! 
//! this structure is basically a lightweight wrapper over a simple `[T; N]` array type.
//! 
//! ## examples
//! 
//! ```
//! # use nyarray::array::Array;
//! let mut array = Array::<16, _>::new(); // new array with capacity of 16
//! 
//! // `Array` functions very similarly to `Vec`.
//! 
//! array.push(1);
//! array.push(2);
//!
//! assert_eq!(array.len(), 2);
//! assert_eq!(array[0], 1);
//! 
//! assert_eq!(array.pop(), Some(2));
//! assert_eq!(array.len(), 1);
//! 
//! array[0] = 7;
//! assert_eq!(array[0], 7);
//! 
//! array.extend([1, 2, 3]);
//! 
//! for x in &array {
//!     println!("{x}");
//! }
//! 
//! assert_eq!(array, [7, 1, 2, 3]);
//! ```
//! 
//! note that, while the terminology "stack-allocated" is used here, one can
//! easily allocate this structure onto the heap like so:
//! 
//! ```
//! # use nyarray::array::Array;
//! # use std::boxed::Box;
//! let array = Box::new(Array::<16, ()>::new());
//! ```
//! 
//! of course, at this point, one should consider using `Vec` or similar.

/// stack-allocated array. see [module level documentation](self) for more.
pub struct Array<const N: usize, T> {
	buf: [core::mem::MaybeUninit<T>; N],
	len: usize,
}

impl<const N: usize, T> Array<N, T> {
	/// create a new [`Array`].
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array::Array;
	/// let array = Array::<16, ()>::new(); // array with capacity of 16
	/// ```
	#[inline]
	pub fn new() -> Self {
		Self {
			buf: [const { core::mem::MaybeUninit::uninit() }; N],
			len: 0,
		}
	}

	/// construct an array from a possibly uninitialized array.
	/// 
	/// ## safety
	/// 
	/// `buf[0..len]` must be fully initialized memory.
	/// 
	/// ## examples
	/// 
	/// this can be useful in combination with [`Self::into_parts_len()`] to
	/// reconstruct the array after taking it apart for whatever reason.
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::array::Array;
	/// let array = array![1, 2, 3 => 3];
	/// 
	/// let (buf, len) = array.into_parts_len();
	/// 
	/// // do whatever to `buf`
	/// 
	/// let array = unsafe { Array::from_parts_len(buf, len) };
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn from_parts_len(buf: [core::mem::MaybeUninit<T>; N], len: usize) -> Self {
		assert!(len <= N);

		Self {
			buf,
			len,
		}
	}

	/// construct an array from an initialized array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array::Array;
	/// let array = Array::<16, i32>::from_parts([1, 2, 3, 4]);
	/// assert_eq!(array, [1, 2, 3, 4]);
	/// ```
	/// 
	/// ## panics
	/// 
	/// this method panics if the const parameter `M` is larger than the
	/// array capacity (const parameter `N`).
	/// 
	/// ```should_panic
	/// # use nyarray::array::Array;
	/// // note the input array is larger than the array capacity
	/// let array = Array::<4, i32>::from_parts([1, 2, 3, 4, 5, 6]);
	/// // this panics!
	/// ```
	#[inline]
	pub fn from_parts<const M: usize>(buf: [T; M]) -> Self {
		assert!(M <= N);

		let buf = core::mem::ManuallyDrop::new(buf);

		let mut new_buf = [const { core::mem::MaybeUninit::uninit() }; N];

		let buf_ptr = buf.as_ptr();
		let new_ptr = new_buf.as_mut_ptr();

		unsafe {
			core::ptr::copy_nonoverlapping(buf_ptr, new_ptr as *mut T, M);

			Self::from_parts_len(new_buf, M)
		}
	}

	/// construct an array from a raw pointer.
	/// 
	/// ## safety
	/// 
	/// - `ptr` must point to memory valid for reads of `len` elements.
	/// - `ptr` must be aligned.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array::Array;
	/// # use std::vec;
	/// let mut vec = vec![1, 2, 3];
	/// let array;
	/// 
	/// unsafe {
	///     let ptr = vec.as_ptr();
	///     let len = vec.len();
	///     vec.set_len(0);
	///     array = Array::<4, _>::from_raw_parts(ptr, len);
	/// }
	/// 
	/// assert_eq!(array, [1, 2, 3]);
	/// ``` 
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn from_raw_parts(ptr: *const T, len: usize) -> Self {
		let mut new_buf = [const { core::mem::MaybeUninit::uninit() }; N];

		let new_ptr = new_buf.as_mut_ptr();

		unsafe {
			core::ptr::copy_nonoverlapping(ptr, new_ptr as *mut T, len);

			Self::from_parts_len(new_buf, len)
		}
	}

	/// deconstruct an array.
	/// 
	/// note that, let `ret` be the output, `ret.0[0..ret.1]` is valid memory. if
	/// `T` is `Drop`, then forgetting to drop this will leak memory.
	/// 
	/// the easiest way to correctly drop this is to reconstruct the array with [`Self::from_parts_len()`].
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::array::Array;
	/// let array = array![1, 2, 3 => 3];
	/// 
	/// let (buf, len) = array.into_parts_len();
	/// 
	/// // do whatever to `buf`
	/// 
	/// let array = unsafe { Array::from_parts_len(buf, len) };
	/// ```
	#[inline]
	pub fn into_parts_len(self) -> ([core::mem::MaybeUninit<T>; N], usize) {
		let this = core::mem::ManuallyDrop::new(self);
		let buf = unsafe {
			core::ptr::read(&this.buf)
		};
		(buf, this.len)
	}

	/// returns the total number of elements the array can hold.
	/// this function always returns the const `N` parameter of this array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let array = array![1, 2, 3 => 16];
	/// assert_eq!(array.capacity(), 16);
	/// ```
	#[inline]
	pub fn capacity(&self) -> usize {
		N
	}

	/// returns the total number of elements inside the array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let array = array![1, 2, 3 => 16];
	/// assert_eq!(array.len(), 3);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		self.len
	}

	/// set the length of the array to `new_len`.
	/// 
	/// ## safety
	/// 
	/// this function should be used with care, as setting `new_len` to incorrect values
	/// can easily expose safe code to uninitialized memory.
	/// 
	/// - `new_len` lesser or equal to [`Self::capacity()`]
	/// - all elements `0..new_len` must be initialized.
	/// 
	/// consider using other safe functions, like [`Self::clear()`] or [`Self::extend()`].
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::array::Array;
	/// unsafe fn fill(input_ptr: *const u32, input_len: usize) -> Array<16, u32> {
	///     // it is UB to copy more than the Array capacity (16)
	///     assert!(input_len <= 16);
	///     let mut array = array![];
	///     let array_ptr = array.as_mut_ptr();
	///     unsafe {
	///         core::ptr::copy(input_ptr, array_ptr, input_len);
	///         // set_len *after* copying input
	///         array.set_len(input_len);
	///     }
	///     array
	/// }
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn set_len(&mut self, new_len: usize) {
		self.len = new_len;
	}

	/// returns `true` if the array has zero elements, `false` otherwise.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::array::Array;
	/// let array: Array<_, ()> = array![=> 4];
	/// assert!(array.is_empty());
	/// ```
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// returns a slice containing the array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::array::Array;
	/// let array: Array<_, u8> = array![=> 4];
	/// let slice: &[u8] = array.as_slice();
	/// // let slice: &[u8] = &array[..]; // works the same
	/// 
	/// let string = str::from_utf8(slice);
	/// ```
	#[inline]
	pub fn as_slice(&self) -> &[T] {
		let out = &self.buf[0..self.len];
		// safety: all elements before `len` should always be initialized
		unsafe {
			core::mem::transmute::<&[core::mem::MaybeUninit<T>], &[T]>(out)
		}
	}

	/// returns a mutable slice containing the array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::array::Array;
	/// let mut array: Array<_, u8> = array![=> 4];
	/// let mut slice: &mut [u8] = array.as_mut_slice();
	/// // let mut slice: &mut [u8] = &mut array[..]; // works the same
	/// 
	/// let string = str::from_utf8_mut(slice);
	/// ```
	#[inline]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		let out = &mut self.buf[0..self.len];
		// safety: all elements before `len` should always be initialized
		unsafe {
			core::mem::transmute::<&mut [core::mem::MaybeUninit<T>], &mut [T]>(out)
		}
	}

	/// returns a raw pointer to the internal buffer.
	/// 
	/// this pointer is valid so long as this array is valid. if the array is
	/// dropped, or even moved, the pointer is immediately invalid.
	#[inline]
	pub fn as_ptr(&self) -> *const T {
		self.buf.as_ptr() as *const T
	}

	/// returns a mutable raw pointer to the internal buffer.
	/// 
	/// this pointer is valid so long as this array is valid. if the array is
	/// dropped, or even moved, the pointer is immediately invalid.
	#[inline]
	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.buf.as_mut_ptr() as *mut T
	}

	/// removes all elements from the array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 4];
	/// array.clear();
	/// assert!(array.is_empty());
	/// ```
	#[inline]
	pub fn clear(&mut self) {
		unsafe {
			let elements = self.as_mut_slice() as *mut [T];
			core::ptr::drop_in_place(elements);
			self.set_len(0);
		}
	}

	/// add an element to the end of the array.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![=> 4];
	/// array.push(1);
	/// array.push(2);
	/// array.push(3);
	/// assert_eq!(array, [1, 2, 3]);
	/// ```
	/// 
	/// ## panics
	/// 
	/// this method panics if there isn't enough space for another element.
	/// for a non-panicking version, see [`Self::push_checked()`].
	/// 
	/// ```should_panic
	/// # use nyarray::array;
	/// let mut array = array![=> 4];
	/// array.push(1);
	/// array.push(2);
	/// array.push(3);
	/// array.push(4);
	/// array.push(5); // panics
	/// ```
	#[inline]
	pub fn push(&mut self, value: T) {
		if self.push_checked(value).is_err() {
			panic!("push exceeds capacity");
		}
	}

	/// add an element to the end of the array. returns `Err(T)` if
	/// there is not enough capacity.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # fn main() -> Result<(), i32> {
	/// # use nyarray::array;
	/// let mut array = array![=> 4];
	/// array.push_checked(1)?;
	/// array.push_checked(2)?;
	/// array.push_checked(3)?;
	/// assert_eq!(array, [1, 2, 3]);
	/// # Ok(())
	/// # }
	/// ```
	#[inline]
	pub fn push_checked(&mut self, value: T) -> Result<(), T> {
		if self.len() == self.capacity() {
			Err(value)
		} else {
			unsafe {
				// safety: just confirmed there is enough space for another element
				self.push_unchecked(value);
			}
			Ok(())
		}
	}

	/// add an element to the end of the array.
	/// 
	/// this is the unsafe version of this method. see [`Self::push()`] or
	/// [`Self::push_checked()`] for safe versions of this.
	/// 
	/// ## safety
	/// 
	/// there must be enough capacity in the array for at least one more element
	/// before calling this method. ie; [`Self::len()`] `<` [`Self::capacity()`].
	/// 
	/// ## examples
	/// 
	/// ```
	/// # fn main() -> Result<(), i32> {
	/// # use nyarray::array;
	/// let mut array = array![=> 4];
	/// unsafe {
	///     // safety: array has capacity of 4 elements.
	///     array.push_unchecked(1);
	///     array.push_unchecked(2);
	///     array.push_unchecked(3);
	///     array.push_unchecked(4);
	///    // array.push_unchecked(5); // UB
	/// }
	/// assert_eq!(array, [1, 2, 3, 4]);
	/// # Ok(())
	/// # }
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn push_unchecked(&mut self, value: T) {
		unsafe {
			let len = self.len();

			// safety: caller ensures there is enough space for another element
			let ptr = self.as_mut_ptr().add(len);

			core::ptr::write(ptr, value);

			// set length to accomodate for new element
			self.set_len(len + 1);
		}
	}

	/// remove and return an element from the end of the array.
	/// returns `None` if the array is empty.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 4];
	/// assert_eq!(array.pop(), Some(3));
	/// assert_eq!(array.pop(), Some(2));
	/// assert_eq!(array.pop(), Some(1));
	/// assert_eq!(array.pop(), None);
	/// ```
	#[inline]
	pub fn pop(&mut self) -> Option<T> {
		if self.is_empty() {
			None
		} else {
			unsafe {
				// safety: just confirmed there is an element in the array
				Some(self.pop_unchecked())
			}
		}
	}

	/// remove and return an element from the end of the array.
	/// 
	/// this is the unsafe version of this method. see [`Self::pop()`] for
	/// the safe version.
	/// 
	/// ## safety
	/// 
	/// there must be at least one element in the array prior to calling
	/// this method. ie; [`Self::len()`] `!= 0`
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 4];
	/// 
	/// unsafe {
	///     // safety: array has 3 elements
	///     assert_eq!(array.pop_unchecked(), 3);
	///     assert_eq!(array.pop_unchecked(), 2);
	///     assert_eq!(array.pop_unchecked(), 1);
	///     // array.pop_unchecked() // UB
	/// }
	/// 
	/// assert!(array.is_empty());
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn pop_unchecked(&mut self) -> T {
		unsafe {
			// safety: caller ensures there is at least one element.

			// underflows if no elements
			let len = self.len() - 1;

			// first set len to new len
			self.set_len(len);

			core::ptr::read(self.as_ptr().add(len))
		}
	}

	/// insert an element into any index of the array, shifting
	/// all elements after towards the end.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 6];
	/// 
	/// array.insert(2, 10);
	/// assert_eq!(array, [1, 2, 10, 3]);
	/// 
	/// array.insert(0, 20);
	/// assert_eq!(array, [20, 1, 2, 10, 3]);
	/// 
	/// array.insert(5, 30);
	/// assert_eq!(array, [20, 1, 2, 10, 3, 30]);
	/// ```
	///  
	/// ## panics
	/// 
	/// this method panics if there isn't enough space for another element,
	/// or if `index` is not `0..self.len()`.
	/// for a non-panicking version, see [`Self::insert_checked()`].
	/// 
	/// ```should_panic
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 4];
	/// array.insert(0, 4); // okay
	/// array.insert(0, 5); // panics
	/// ```
	#[inline]
	pub fn insert(&mut self, index: usize, element: T) {
		if self.insert_checked(index, element).is_err() {
			if index > self.len() {
				panic!("index out of bounds");
			} else {
				panic!("insert exceeds capacity");
			}
		}
	}

	/// insert an element into any index of the array, shifting
	/// all elements after towards the end. returns Err(T) if there
	/// is not enough capacity, or if `index` is not `0..=self.len()`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # fn main() -> Result<(), i32> {
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 6];
	/// 
	/// array.insert_checked(2, 10)?;
	/// assert_eq!(array, [1, 2, 10, 3]);
	/// 
	/// array.insert_checked(0, 20)?;
	/// assert_eq!(array, [20, 1, 2, 10, 3]);
	/// 
	/// array.insert_checked(5, 30)?;
	/// assert_eq!(array, [20, 1, 2, 10, 3, 30]);
	/// # Ok(())
	/// # }
	/// ```
	#[inline]
	pub fn insert_checked(&mut self, index: usize, element: T) -> Result<(), T> {
		if index > self.len() {
			return Err(element);
		}

		if self.len() + 1 > self.capacity() {
			return Err(element);
		}

		unsafe {
			// safety: just confirmed index is in bounds and there is enough capacity
			self.insert_unchecked(index, element);
		}

		Ok(())
	}

	/// insert an element into any index of the array, shifting
	/// all elements after towards the end.
	/// 
	/// this is the unsafe version of this method. see [`Self::insert_checked()`] or
	/// [`Self::insert()`] for safe versions.
	/// 
	/// ## safety
	/// 
	/// - there must be enough capacity in the array for at least one more element
	///   prior to calling this method. ie; [`Self::len()`] `<` [`Self::capacity()`].
	/// - `index` `<=` [`Self::len()`]
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![=> 4];
	/// 
	/// unsafe {
	///     // safety: array has capacity of 4 elements.
	///     array.insert_unchecked(0, 1);
	///     array.insert_unchecked(0, 2);
	///     array.insert_unchecked(0, 3);
	///     array.insert_unchecked(0, 4);
	///     // array.insert_unchecked(0, 5); // UB
	/// }
	/// 
	/// assert_eq!(array, [4, 3, 2, 1]);
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn insert_unchecked(&mut self, index: usize, element: T) {
		unsafe {
			let len = self.len();

			let ptr = self.as_mut_ptr().add(index);

			if index != len {
				core::ptr::copy(ptr, ptr.add(1), len - index);
			}

			core::ptr::write(ptr, element);

			self.set_len(len + 1);
		}
	}

	/// insert an element into any index of the array, moving the element
	/// that was previously there to the end.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 6];
	/// 
	/// array.swap_insert(2, 10);
	/// assert_eq!(array, [1, 2, 10, 3]);
	/// 
	/// array.swap_insert(0, 20);
	/// assert_eq!(array, [20, 2, 10, 3, 1]);
	/// 
	/// array.swap_insert(5, 30);
	/// assert_eq!(array, [20, 2, 10, 3, 1, 30]);
	/// ```
	///  
	/// ## panics
	/// 
	/// this method panics if there isn't enough space for another element,
	/// or if `index` is not `0..=self.len()`.
	/// for a non-panicking version, see [`Self::swap_insert_checked()`].
	/// 
	/// ```should_panic
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 4];
	/// array.swap_insert(0, 4); // okay
	/// array.swap_insert(0, 5); // panics
	/// ```
	#[inline]
	pub fn swap_insert(&mut self, index: usize, element: T) {
		if self.swap_insert_checked(index, element).is_err() {
			if index > self.len() {
				panic!("index out of bounds");
			} else {
				panic!("insert exceeds capacity");
			}
		}
	}

	/// insert an element into any index of the array, moving the element
	/// that was previously there to the end. returns Err(T) if there
	/// is not enough capacity, or if `index` is not `0..=self.len()`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # fn main() -> Result<(), i32> {
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3 => 6];
	/// 
	/// array.swap_insert_checked(2, 10)?;
	/// assert_eq!(array, [1, 2, 10, 3]);
	/// 
	/// array.swap_insert_checked(0, 20)?;
	/// assert_eq!(array, [20, 2, 10, 3, 1]);
	/// 
	/// array.swap_insert_checked(5, 30)?;
	/// assert_eq!(array, [20, 2, 10, 3, 1, 30]);
	/// # Ok(())
	/// # }
	/// ```
	#[inline]
	pub fn swap_insert_checked(&mut self, index: usize, element: T) -> Result<(), T> {
		if index > self.len() {
			return Err(element);
		}

		if self.len() + 1 > self.capacity() {
			return Err(element);
		}

		unsafe {
			// safety: just confirmed index is in bounds and there is enough capacity
			self.swap_insert_unchecked(index, element);
		}

		Ok(())
	}

	/// insert an element into any index of the array, moving the element
	/// that was previously there to the end.
	/// 
	/// this is the unsafe version of this method. see [`Self::swap_insert_checked()`]
	/// or [`Self::swap_insert()`] for safe versions.
	/// 
	/// ## safety
	/// 
	/// - there must be enough capacity in the array for at least one more element
	///   prior to calling this method. ie; [`Self::len()`] `<` [`Self::capacity()`].
	/// - `index` `<=` [`Self::len()`]
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![=> 4];
	/// 
	/// unsafe {
	///     // safety: array has a capacity of 4
	///     array.swap_insert_unchecked(0, 1);
	///     array.swap_insert_unchecked(0, 2);
	///     array.swap_insert_unchecked(0, 3);
	///     array.swap_insert_unchecked(0, 4);
	///     // array.swap_insert_unchecked(0, 5); // UB
	/// }
	/// 
	/// assert_eq!(array, [4, 1, 2, 3])
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn swap_insert_unchecked(&mut self, index: usize, element: T) {
		unsafe {
			let len = self.len();

			let ptr = self.as_mut_ptr();

			// safety: caller ensures `index` is in bounds and there is enough
			// space for another element.
			let old_ptr = ptr.add(index);
			let new_ptr = ptr.add(len);

			core::ptr::write(new_ptr, element);
			core::ptr::swap(old_ptr, new_ptr);
			
			self.set_len(len + 1);
		}
	}

	/// remove and return an element out of any index of the array,
	/// shifting all elements after towards the start.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4, 5, 6 => 6];
	/// 
	/// assert_eq!(array.remove(0), 1);
	/// assert_eq!(array, [2, 3, 4, 5, 6]);
	/// 
	/// assert_eq!(array.remove(2), 4);
	/// assert_eq!(array, [2, 3, 5, 6]);
	/// 
	/// assert_eq!(array.remove(3), 6);
	/// assert_eq!(array, [2, 3, 5]);
	/// ```
	/// 
	/// ## panics
	/// 
	/// this method panics if `index` is not `0..self.len()`.
	/// for a non-panicking version, see [`Self::remove_checked()`].
	/// 
	/// ```should_panic
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4 => 4];
	/// array.remove(4); // panics
	/// ```
	#[inline]
	pub fn remove(&mut self, index: usize) -> T {
		if let Some(x) = self.remove_checked(index) {
			x
		} else {
			panic!("index out of bounds");
		}
	}

	/// remove and return an element out of any index of the array,
	/// shifting all elements after towards the start. returns `None`
	/// if `index` is not `0..self.len()`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4, 5, 6 => 6];
	/// 
	/// assert_eq!(array.remove_checked(0), Some(1));
	/// assert_eq!(array, [2, 3, 4, 5, 6]);
	/// 
	/// assert_eq!(array.remove_checked(2), Some(4));
	/// assert_eq!(array, [2, 3, 5, 6]);
	/// 
	/// assert_eq!(array.remove_checked(3), Some(6));
	/// assert_eq!(array, [2, 3, 5]);
	/// 
	/// assert_eq!(array.remove_checked(3), None);
	/// assert_eq!(array, [2, 3, 5]);
	/// ```
	#[inline]
	pub fn remove_checked(&mut self, index: usize) -> Option<T> {
		if index >= self.len() {
			return None;
		}

		unsafe {
			Some(self.remove_unchecked(index))
		}
	}

	/// remove and return an element out of any index of the array,
	/// shifting all elements after towards the start.
	/// 
	/// this is the unsafe version of this method. see [`Self::remove_checked()`]
	/// or [`Self::remove()`] for safe versions.
	/// 
	/// ## safety
	/// 
	/// - there must be at least one element in the array prior to calling
	///   this method. ie; [`Self::len()`] `!= 0`
	/// - `index` `<` [`Self::len()`]
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4 => 4];
	/// 
	/// unsafe {
	///     // safety: array has 4 elements.
	///     assert_eq!(array.remove_unchecked(0), 1);
	///     assert_eq!(array.remove_unchecked(0), 2);
	///     assert_eq!(array.remove_unchecked(0), 3);
	///     assert_eq!(array.remove_unchecked(0), 4);
	///     // array.remove_unchecked(0) // UB
	/// }
	/// 
	/// assert!(array.is_empty());
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {		
		unsafe {
			let len = self.len();

			// safety: caller ensures index is in bounds and there is at least one element
			let ptr = self.as_mut_ptr().add(index);

			let old = core::ptr::read(ptr);

			core::ptr::copy(ptr.add(1), ptr, len - index - 1);

			self.set_len(len - 1);

			old
		}
	}

	/// remove and return an element from any index of the array,
	/// moving the element that was previously at the end to there.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4, 5, 6 => 6];
	/// 
	/// assert_eq!(array.swap_remove(0), 1);
	/// assert_eq!(array, [6, 2, 3, 4, 5]);
	/// 
	/// assert_eq!(array.swap_remove(2), 3);
	/// assert_eq!(array, [6, 2, 5, 4]);
	/// 
	/// assert_eq!(array.swap_remove(3), 4);
	/// assert_eq!(array, [6, 2, 5]);
	/// ```
	/// 
	/// ## panics
	/// 
	/// this method panics if `index` is not `0..=self.len()`.
	/// for a non-panicking version, see [`Self::swap_remove_checked()`].
	/// 
	/// ```should_panic
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4 => 4];
	/// array.swap_remove(4); // panics
	/// ```
	#[inline]
	pub fn swap_remove(&mut self, index: usize) -> T {
		if let Some(x) = self.swap_remove_checked(index) {
			x
		} else {
			panic!("index out of bounds");
		}
	}

	/// remove and return an element from any index of the array,
	/// moving the element that was previously at the end to there.
	/// returns `None` if `index` is not `0..self.len()`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4, 5, 6 => 6];
	/// 
	/// assert_eq!(array.swap_remove_checked(0), Some(1));
	/// assert_eq!(array, [6, 2, 3, 4, 5]);
	/// 
	/// assert_eq!(array.swap_remove_checked(2), Some(3));
	/// assert_eq!(array, [6, 2, 5, 4]);
	/// 
	/// assert_eq!(array.swap_remove_checked(3), Some(4));
	/// assert_eq!(array, [6, 2, 5]);
	/// 
	/// assert_eq!(array.swap_remove_checked(3), None);
	/// assert_eq!(array, [6, 2, 5]);
	/// ```
	#[inline]
	pub fn swap_remove_checked(&mut self, index: usize) -> Option<T> {
		let len = self.len();
		if index >= len {
			return None;
		}

		unsafe {
			Some(self.swap_remove_unchecked(index))
		}
	}

	/// remove and return an element from any index of the array,
	/// moving the element that was previously at the end to there.
	/// returns `None` if `index` is not `0..self.len()`.
	/// 
	/// this is the unsafe version of this method. see [`Self::swap_remove_checked()`]
	/// or [`Self::swap_remove()`] for safe versions.
	/// 
	/// ## safety
	/// 
	/// - there must be at least one element in the array prior to calling
	///   this method. ie; [`Self::len()`] `!= 0`
	/// - `index` `<` [`Self::len()`]
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// let mut array = array![1, 2, 3, 4 => 4];
	/// 
	/// unsafe {
	///     // safety: array has 4 elements.
	///     assert_eq!(array.swap_remove_unchecked(0), 1);
	///     assert_eq!(array.swap_remove_unchecked(0), 4);
	///     assert_eq!(array.swap_remove_unchecked(0), 3);
	///     assert_eq!(array.swap_remove_unchecked(0), 2);
	///     // array.swap_remove_unchecked(0) // UB
	/// }
	/// 
	/// assert!(array.is_empty());
	/// ```
	#[inline]
	#[expect(clippy::missing_safety_doc, reason = "there is a safety doc, not sure why the lint still generates")]
	pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
		unsafe {
			let len = self.len();

			let ptr = self.as_mut_ptr();
			
			// safety: caller ensures index is in bounds and there is at least one element
			let old = core::ptr::read(ptr.add(index));
			
			core::ptr::copy(ptr.add(len - 1), ptr.add(index), 1);
			
			self.set_len(len - 1);
			
			old
		}
	}
}

impl<const N: usize, T> Drop for Array<N, T> {
	fn drop(&mut self) {
		self.clear();
	}
}

impl<const N: usize, T> Default for Array<N, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<const N: usize, T: Clone> Clone for Array<N, T> {
	fn clone(&self) -> Self {
		self.iter().cloned().collect()
	}
}

impl<const N: usize, T> AsRef<[T]> for Array<N, T> {
	fn as_ref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<const N: usize, T> AsMut<[T]> for Array<N, T> {
	fn as_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<const N: usize, T> core::borrow::Borrow<[T]> for Array<N, T> {
	fn borrow(&self) -> &[T] {
		self.as_slice()
	}
}

impl<const N: usize, T> core::borrow::BorrowMut<[T]> for Array<N, T> {
	fn borrow_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<const N: usize, T> core::ops::Deref for Array<N, T> {
	type Target = [T];
	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl<const N: usize, T> core::ops::DerefMut for Array<N, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_slice()
	}
}

impl<const N: usize, T, I: core::slice::SliceIndex<[T]>> core::ops::Index<I> for Array<N, T> {
	type Output = I::Output;
	fn index(&self, index: I) -> &Self::Output {
		core::ops::Index::index(self.as_slice(), index)
	}
}

impl<const N: usize, T, I: core::slice::SliceIndex<[T]>> core::ops::IndexMut<I> for Array<N, T> {
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		core::ops::IndexMut::index_mut(self.as_mut_slice(), index)
	}
}

impl<const N: usize, T> Extend<T> for Array<N, T> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		for i in iter {
			if self.push_checked(i).is_err() {
				break;
			}
		}
	}
}

impl<'a, const N: usize, T: Copy> Extend<&'a T> for Array<N, T> {
	fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
		for i in iter {
			if self.push_checked(*i).is_err() {
				break;
			}
		}
	}
}


#[doc(hidden)]
pub fn from_elem<const N: usize, T: Clone>(elem: T, n: usize) -> Array<N, T> {
	let mut array = Array::new();
	for _ in 0..n {
		array.push(elem.clone());
	}
	array
}


/// iterator for [`Array`].
pub struct IntoIter<const N: usize, T> {
	inner: [core::mem::MaybeUninit<T>; N],
	cur: usize,
	end: usize,
}

impl<const N: usize, T> Drop for IntoIter<N, T> {
	fn drop(&mut self) {
		while self.cur != self.end {
			unsafe {
				self.inner.get_unchecked_mut(self.cur).assume_init_drop();
				self.cur += 1;
			}
		}
	}
}

impl<const N: usize, T> Iterator for IntoIter<N, T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.cur == self.end {
			return None;
		}
		unsafe {
			let out = self.inner.get_unchecked(self.cur).assume_init_read();
			self.cur += 1;
			Some(out)
		}
	}
}

impl<const N: usize, T> DoubleEndedIterator for IntoIter<N, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.cur == self.end {
			return None;
		}
		unsafe {
			self.end -= 1;
			Some(self.inner.get_unchecked(self.end).assume_init_read())
		}
	}
}

impl<const N: usize, T> IntoIterator for Array<N, T> {
	type IntoIter = IntoIter<N, T>;
	type Item = T;
	
	fn into_iter(self) -> Self::IntoIter {
		let (buf, len) = self.into_parts_len();
		IntoIter {
			inner: buf,
			cur: 0,
			end: len,
		}
	}
}

impl<'a, const N: usize, T> IntoIterator for &'a Array<N, T> {
	type IntoIter = core::slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.as_slice().iter()
	}
}

impl<'a, const N: usize, T> IntoIterator for &'a mut Array<N, T> {
	type IntoIter = core::slice::IterMut<'a, T>;
	type Item = &'a mut T;

	fn into_iter(self) -> Self::IntoIter {
		self.as_mut_slice().iter_mut()
	}
}

impl<const N: usize, T> FromIterator<T> for Array<N, T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut out = Self::new();
		out.extend(iter);
		out
	}
}


impl<const N: usize, T: PartialOrd> PartialOrd for Array<N, T> {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, T: Eq> Eq for Array<N, T> {}

impl<const N: usize, T: Ord> Ord for Array<N, T> {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		Ord::cmp(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, const M: usize, T: PartialEq> PartialEq<Array<M, T>> for Array<N, T> {
	fn eq(&self, other: &Array<M, T>) -> bool {
		PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, T: PartialEq> PartialEq<&[T]> for Array<N, T> {
	fn eq(&self, other: &&[T]) -> bool {
		PartialEq::eq(self.as_slice(), *other)
	}
}

impl<const N: usize, T: PartialEq> PartialEq<&mut [T]> for Array<N, T> {
	fn eq(&self, other: &&mut [T]) -> bool {
		PartialEq::eq(self.as_slice(), *other)
	}
}

impl<const N: usize, const M: usize, T: PartialEq> PartialEq<[T; M]> for Array<N, T> {
	fn eq(&self, other: &[T; M]) -> bool {
		PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, const M: usize, T: PartialEq> PartialEq<&[T; M]> for Array<N, T> {
	fn eq(&self, other: &&[T; M]) -> bool {
		PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, T: core::fmt::Debug> core::fmt::Debug for Array<N, T> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		core::fmt::Debug::fmt(self.as_slice(), f)
	}
}


/// create an [`Array`].
/// 
/// like `vec!`, `array!` has similar syntax as Rust array expressions, with
/// the addition of allowing one to specify the capacity of the `Array`
/// by appending an `=>`:
/// 
/// ```
/// # use nyarray::array;
/// let array = array![1, 2, 3 => 6]; // capacity of 6 elements
/// assert_eq!(array[0], 1);
/// assert_eq!(array[1], 2);
/// assert_eq!(array[2], 3);
/// ```
#[macro_export]
macro_rules! array {
	() => {
		$crate::array::Array::new()
	};
	(=> $cap:literal) => {
		$crate::array::Array::<$cap, _>::new()
	};
	($elem:expr; $n:expr) => {
		$crate::array::from_elem($elem, $n)
	};
	($elem:expr; $n:expr => $cap:literal) => {
		$crate::array::from_elem::<$cap, _>($elem, $n)
	};
	($($x:expr),+ $(,)?) => {
		$crate::array::Array::from_parts([$($x),+])
	};
	($($x:expr),+ $(,)? => $cap:literal) => {
		$crate::array::Array::<$cap, _>::from_parts([$($x),+])
	};
}


#[cfg(test)]
mod test {
	extern crate std;

	#[test]
	fn test_drop() {
		static mut NUM: u32 = 0;

		struct Box<T> {
			_inner: std::boxed::Box<T>,
		}
		impl<T> Box<T> {
			fn new(inner: T) -> Self {
				Self {
					_inner: std::boxed::Box::new(inner),
				}
			}
		}
		impl<T> Drop for Box<T> {
			fn drop(&mut self) {
				unsafe {
					NUM += 1;
				}
			}
		}

		let array = array![Box::new(1), Box::new(2), Box::new(3) => 4];
		
		drop(array);
		
		assert_eq!(unsafe { NUM }, 3);
		
		let array = array![Box::new(1), Box::new(2), Box::new(3) => 4];
		let mut iter = array.into_iter();
		iter.next();
		
		drop(iter);

		assert_eq!(unsafe { NUM }, 6);
	}

	#[test]
	fn test_iter() {
		let array = array![std::boxed::Box::new(1) => 4];
		let _ = array.iter().cloned().collect::<crate::array::Array<4, _>>();
	}
}

