//! a combination of the stack-allocated [`crate::array::Array`] and the heap-allocated `Vec`.
//! 
//! since this combines the functionality of both structures, the public api ends up being
//! something of a union of what either can safely do, meaning there's a lot of functionality
//! either `Vec` and `Array` individually have that `SwitchVec` doesn't.
//! 
//! ## examples
//! 
//! ```
//! # use nyarray::switch::SwitchVec;
//! let mut vec = SwitchVec::<16, _>::new(); // new vec with capacity of 16
//! 
//! // `SwitchVec` functions very similarly to `Vec`.
//! 
//! vec.push(1).unwrap();
//! vec.push(2).unwrap();
//!
//! assert_eq!(vec.len(), 2);
//! assert_eq!(vec[0], 1);
//! 
//! assert_eq!(vec.pop(), Some(2));
//! assert_eq!(vec.len(), 1);
//! 
//! vec[0] = 7;
//! assert_eq!(vec[0], 7);
//! 
//! vec.extend([1, 2, 3]);
//! 
//! for x in &vec {
//!     println!("{x}");
//! }
//! 
//! assert_eq!(vec, [7, 1, 2, 3]);
//! ```
//! 
//! the differentiating detail here is that, by default, `SwitchVec` is stack-allocated.
//! if the `std` feature is enabled, when its capacity is reached, it allocates on the heap.
//! 
//! ```
//! # use nyarray::switch::SwitchVec;
//! let mut vec = SwitchVec::<4, _>::new();
//! 
//! assert!(!vec.is_heap());
//! 
//! vec.push(1).unwrap();
//! vec.push(2).unwrap();
//! vec.push(3).unwrap();
//! vec.push(4).unwrap();
//! 
//! assert!(!vec.is_heap());
//! 
//! vec.push(5).unwrap();
//! 
//! assert!(vec.is_heap());
//! ```


enum Inner<const N: usize, T> {
	Stack(crate::array::Array<N, T>),
	#[cfg(feature = "std")]
	Heap(std::vec::Vec<T>),
}

/// see the [module level documentation](self).
pub struct SwitchVec<const N: usize, T> {
	inner: Inner<N, T>,
}

impl<const N: usize, T> SwitchVec<N, T> {
	/// construct a new [`SwitchVec`]. by default, it will be stack-allocated.
	/// call [`Self::switch_heap()`] to switch to heap-allocation.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// let vec = SwitchVec::<4, ()>::new();
	/// ```
	#[inline]
	pub fn new() -> Self {
		Self {
			inner: Inner::Stack(crate::array::Array::new())
		}
	}

	/// construct a [`SwitchVec`] from a `Vec`.
	/// 
	/// this method is not available in `no_std`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use std::vec;
	/// let vec = SwitchVec::<4, _>::from_vec(vec![0, 1, 2]);
	/// ```
	#[cfg(feature = "std")]
	#[inline]
	pub fn from_vec(vec: std::vec::Vec<T>) -> Self {
		Self {
			inner: Inner::Heap(vec)
		}
	}

	/// deconstruct this vec into a `Vec`, or `Err` if [`Self::is_heap()`] is `false`.
	/// 
	/// this method is not available in `no_std`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use std::vec;
	/// let vec = SwitchVec::<4, _>::from_vec(vec![0, 1, 2]);
	/// ```
	#[cfg(feature = "std")]
	#[inline]
	pub fn into_vec(self) -> Result<std::vec::Vec<T>, Self> {
		match self.inner {
			Inner::Stack(..) => Err(self),
			Inner::Heap(vec) => Ok(vec),
		}
	}

	/// construct a [`SwitchVec`] from an [`crate::array::Array`].
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let vec = SwitchVec::from_array(array![0, 1, 2 => 4]);
	/// ```
	#[inline]
	pub fn from_array(array: crate::array::Array<N, T>) -> Self {
		Self {
			inner: Inner::Stack(array)
		}
	}

	/// deconstruct this vec into an `Array`, or `Err` if [`Self::is_heap()`] is `true`.
	#[inline]
	pub fn into_array(self) -> Result<crate::array::Array<N, T>, Self> {
		match self.inner {
			Inner::Stack(array) => Ok(array),
			#[cfg(feature = "std")]
			Inner::Heap(..) => Err(self),
		}
	}

	/// returns the total number of elements the vector can hold without allocating.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![0, 1, 2 => 8]);
	/// 
	/// assert_eq!(vec.capacity(), 8);
	/// 
	/// vec.extend([3, 4, 5, 6, 7]);
	/// 
	/// assert_eq!(vec.capacity(), 8);
	/// 
	/// vec.extend([8, 9]);
	/// 
	/// assert!(vec.capacity() > 8);
	/// ```
	#[inline]
	pub fn capacity(&self) -> usize {
		match &self.inner {
			Inner::Stack(array) => array.capacity(),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.capacity(),
		}
	}

	/// returns the total number of elements inside the vector.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![0, 1, 2 => 8]);
	/// 
	/// assert_eq!(vec.len(), 3);
	/// 
	/// vec.extend([3, 4, 5]);
	/// 
	/// assert_eq!(vec.len(), 6);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		match &self.inner {
			Inner::Stack(array) => array.len(),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.len(),
		}
	}

	/// returns `true` if the vector has zero elements, `false` otherwise.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let vec = SwitchVec::<_, ()>::from_array(array![=> 8]);
	/// assert!(vec.is_empty());
	/// ```
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// returns `true` if the vector is heap-allocated, `false` otherwise.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// # use std::vec;
	/// let vec = SwitchVec::<_, ()>::from_array(array![=> 8]);
	/// 
	/// assert!(!vec.is_heap());
	/// 
	/// let vec = SwitchVec::<8, ()>::from_vec(vec![]);
	/// 
	/// assert!(vec.is_heap());
	/// ```
	#[inline]
	pub fn is_heap(&self) -> bool {
		match &self.inner {
			Inner::Stack(..) => false,
			#[cfg(feature = "std")]
			Inner::Heap(..) => true,
		}
	}

	/// returns a slice containing the vector.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::switch::SwitchVec;
	/// let vector: SwitchVec<_, u8> = SwitchVec::from_array(array![=> 4]);
	/// let slice: &[u8] = vector.as_slice();
	/// // let slice: &[u8] = &vector[..]; // works the same
	/// 
	/// let string = str::from_utf8(slice);
	/// ```
	#[inline]
	pub fn as_slice(&self) -> &[T] {
		match &self.inner {
			Inner::Stack(array) => array,
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec,
		}
	}

	/// returns a mutable slice containing the vector.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::array;
	/// # use nyarray::switch::SwitchVec;
	/// let mut vector: SwitchVec<_, u8> = SwitchVec::from_array(array![=> 4]);
	/// let mut slice: &mut [u8] = vector.as_mut_slice();
	/// // let mut slice: &mut [u8] = &mut vector[..]; // works the same
	/// 
	/// let string = str::from_utf8_mut(slice);
	/// ```
	#[inline]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		match &mut self.inner {
			Inner::Stack(array) => array,
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec,
		}
	}

	/// returns a raw pointer to the internal buffer.
	/// 
	/// if the vector is heap-allocated, this pointer is valid for the lifetime
	/// of the vector. if an operation reallocates, this pointer becomes invalid.
	/// 
	/// if the vector is stack-allocated, this pointer is valid for the lifetime
	/// of the vector, so long as the vector is not moved. if an operation reallocates,
	/// this pointer becomes invalid.
	#[inline]
	pub fn as_ptr(&self) -> *const T {
		match &self.inner {
			Inner::Stack(array) => array.as_ptr(),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.as_ptr(),
		}
	}

	/// returns a mutable raw pointer to the internal buffer.
	/// 
	/// if the vector is heap-allocated, this pointer is valid for the lifetime
	/// of the vector. if an operation reallocates, this pointer becomes invalid.
	/// 
	/// if the vector is stack-allocated, this pointer is valid for the lifetime
	/// of the vector, so long as the vector is not moved. if an operation reallocates,
	/// this pointer becomes invalid.
	#[inline]
	pub fn as_mut_ptr(&mut self) -> *mut T {
		match &mut self.inner {
			Inner::Stack(array) => array.as_mut_ptr(),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.as_mut_ptr(),
		}
	}

	/// removes all elements from the vector.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![1, 2, 3 => 4]);
	/// vec.clear();
	/// assert!(vec.is_empty());
	/// ```
	#[inline]
	pub fn clear(&mut self) {
		match &mut self.inner {
			Inner::Stack(array) => array.clear(),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.clear(),
		}
	}

	/// move this vector's elements onto the heap, if not already done so.
	/// returns `true` if successful.
	/// returns `false` if the operation failed for whatever reason.
	/// 
	/// in `no_std`, this is a no-op, and always returns `false``.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![1, 2, 3 => 4]);
	/// 
	/// assert!(!vec.is_heap());
	/// 
	/// vec.switch_heap();
	/// 
	/// assert!(vec.is_heap());
	/// ```
	#[must_use]
	pub fn switch_heap(&mut self) -> bool {
		#[cfg(feature = "std")]
		{
			let array = match &mut self.inner {
				Inner::Stack(array) => {
					array
				}
				Inner::Heap(..) => {
					return true;
				}
			};

			// create vector first
			let mut vec = std::vec::Vec::new();

			// try allocate; if fails, bail before anything else happens
			if vec.try_reserve_exact(array.len()).is_err() {
				return false;
			}

			// first read array
			let array = unsafe {
				core::ptr::read(array)
			};

			// then write to inner with valid Vec to avoid drop
			unsafe {
				core::ptr::write(
					&mut self.inner,
					Inner::Heap(vec),
				);
			}

			let Inner::Heap(vec) = &mut self.inner else {
				// even if this was reachable, we own `array`, so no UB
				unreachable!();
			};

			// insert array elements into vector
			vec.extend(array);

			true
		}
		#[cfg(not(feature = "std"))]
		{
			false
		}
	}

	/// move this vector's elements onto the heap, if not already done so.
	/// this is a lossy operation - elements that don't fit in the array
	/// will be discarded.
	/// returns `true` if successful.
	/// returns `false` if the operation failed for whatever reason.
	/// 
	/// in `no_std`, this is a no-op, and always returns `true`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use std::vec;
	/// let mut vec = SwitchVec::<4, _>::from_vec(vec![1, 2, 3, 4, 5]);
	/// 
	/// assert!(vec.is_heap());
	/// 
	/// vec.switch_stack();
	/// 
	/// assert!(!vec.is_heap());
	/// 
	/// assert_eq!(vec, [1, 2, 3, 4]);
	/// ```
	#[must_use]
	pub fn switch_stack(&mut self) -> bool {
		#[cfg(feature = "std")]
		{
			let vec = match &mut self.inner {
				Inner::Stack(..) => {
					return true;
				}
				Inner::Heap(vec) => {
					vec
				}
			};

			// first read vec
			let vec = unsafe {
				core::ptr::read(vec)
			};

			// then write to inner with valid Array to avoid drop
			unsafe {
				core::ptr::write(
					&mut self.inner,
					Inner::Stack(crate::array::Array::new()),
				);
			}

			let Inner::Stack(array) = &mut self.inner else {
				// even if this was reachable, we own `array`, so no UB
				unreachable!();
			};

			// insert vector elements into array
			array.extend(vec);

			true
		}
		#[cfg(not(feature = "std"))]
		{
			true
		}
	}

	/// ensure [`Self::capacity()`] has enough space for `additional` number of element.
	/// returns `true` if there is enough space, or if not, memory was successfully allocated.
	/// returns `false` if memory could not be allocated for whatever reason.
	/// 
	/// if [`Self::is_heap()`] is `false` and there isn't enough array capacity, this will
	/// move the vector's elements to the heap.
	/// 
	/// if `no_std`, this is a no-op, and always returns `false`.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use std::vec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::<4, _>::from_array(array![1, 2, 3, 4]);
	/// 
	/// assert_eq!(vec.capacity(), 4);
	/// assert_eq!(vec.len(), 4);
	/// assert!(!vec.is_heap());
	/// 
	/// vec.reserve(4);
	/// 
	/// assert!(vec.capacity() >= 8);
	/// assert_eq!(vec.len(), 4);
	/// assert!(vec.is_heap());
	/// ```
	#[must_use]
	pub fn reserve(&mut self, additional: usize) -> bool {
		#[cfg(feature = "std")]
		{
			match &mut self.inner {
				Inner::Stack(array) => {
					if array.len() + additional <= array.capacity() {
						return true;
					}

					if !self.switch_heap() {
						return false
					}

					let Inner::Heap(vec) = &mut self.inner else {
						unreachable!();
					};
					
					vec.try_reserve(additional).is_ok()
				}
				Inner::Heap(vec) => {
					vec.try_reserve(additional).is_ok()
				}
			}
		}
		#[cfg(not(feature = "std"))]
		{
			_ = additional;
			false
		}
	}

	/// add an element to the end of the vector, returning
	/// `Err(T)` if the operation failed.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![=> 2]);
	/// vec.push(0).unwrap();
	/// vec.push(1).unwrap();
	/// vec.push(2).unwrap();
	/// assert_eq!(vec.len(), 3);
	/// ```
	#[inline]
	pub fn push(&mut self, value: T) -> Result<(), T> {
		if !self.reserve(1) {
			return Err(value);
		}

		match &mut self.inner {
			Inner::Stack(array) => array.push(value),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.push(value),
		}

		Ok(())
	}

	/// remove and return an element from the end of the vector.
	/// returns `None` if the vector is empty.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![0, 1, 2 => 4]);
	/// assert_eq!(vec.pop(), Some(2));
	/// assert_eq!(vec.pop(), Some(1));
	/// assert_eq!(vec.pop(), Some(0));
	/// assert_eq!(vec.pop(), None);
	/// ```
	#[inline]
	pub fn pop(&mut self) -> Option<T> {
		match &mut self.inner {
			Inner::Stack(array) => array.pop(),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.pop(),
		}
	}

	/// insert an element into any index of the vector, shifting
	/// all elements after towards the end.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![1, 2, 3 => 4]);
	/// 
	/// vec.insert(2, 10).unwrap();
	/// assert_eq!(vec, [1, 2, 10, 3]);
	/// 
	/// vec.insert(0, 20).unwrap();
	/// assert_eq!(vec, [20, 1, 2, 10, 3]);
	/// 
	/// vec.insert(5, 30).unwrap();
	/// assert_eq!(vec, [20, 1, 2, 10, 3, 30]);
	/// ```
	#[inline]
	pub fn insert(&mut self, index: usize, element: T) -> Result<(), T> {
		if !self.reserve(1) {
			return Err(element);
		}

		match &mut self.inner {
			Inner::Stack(array) => array.insert(index, element),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => vec.insert(index, element),
		}

		Ok(())
	}

	/// remove and return an element out of any index of the vector,
	/// shifting all elements after towards the start.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![1, 2, 3, 4, 5, 6 => 6]);
	/// 
	/// assert_eq!(vec.remove(0), Some(1));
	/// assert_eq!(vec, [2, 3, 4, 5, 6]);
	/// 
	/// assert_eq!(vec.remove(2), Some(4));
	/// assert_eq!(vec, [2, 3, 5, 6]);
	/// 
	/// assert_eq!(vec.remove(3), Some(6));
	/// assert_eq!(vec, [2, 3, 5]);
	/// 
	/// assert_eq!(vec.remove(3), None);
	/// ```
	#[inline]
	pub fn remove(&mut self, index: usize) -> Option<T> {
		if index >= self.len() {
			return None;
		}

		match &mut self.inner {
			Inner::Stack(array) => Some(array.remove(index)),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => Some(vec.remove(index)),
		}
	}

	/// remove and return an element from any index of the vector,
	/// moving the element that was previously at the end to there.
	/// 
	/// ## examples
	/// 
	/// ```
	/// # use nyarray::switch::SwitchVec;
	/// # use nyarray::array;
	/// let mut vec = SwitchVec::from_array(array![1, 2, 3, 4, 5, 6 => 6]);
	/// 
	/// assert_eq!(vec.swap_remove(0), Some(1));
	/// assert_eq!(vec, [6, 2, 3, 4, 5]);
	/// 
	/// assert_eq!(vec.swap_remove(2), Some(3));
	/// assert_eq!(vec, [6, 2, 5, 4]);
	/// 
	/// assert_eq!(vec.swap_remove(3), Some(4));
	/// assert_eq!(vec, [6, 2, 5]);
	/// 
	/// assert_eq!(vec.swap_remove(3), None);
	/// ```
	#[inline]
	pub fn swap_remove(&mut self, index: usize) -> Option<T> {
		if index >= self.len() {
			return None;
		}

		match &mut self.inner {
			Inner::Stack(array) => Some(array.swap_remove(index)),
			#[cfg(feature = "std")]
			Inner::Heap(vec) => Some(vec.swap_remove(index)),
		}
	}
}

impl<const N: usize, T> Default for SwitchVec<N, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<const N: usize, T: Clone> Clone for SwitchVec<N, T> {
	fn clone(&self) -> Self {
		self.iter().cloned().collect()
	}
}

impl<const N: usize, T> AsRef<[T]> for SwitchVec<N, T> {
	fn as_ref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<const N: usize, T> AsMut<[T]> for SwitchVec<N, T> {
	fn as_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<const N: usize, T> core::borrow::Borrow<[T]> for SwitchVec<N, T> {
	fn borrow(&self) -> &[T] {
		self.as_slice()
	}
}

impl<const N: usize, T> core::borrow::BorrowMut<[T]> for SwitchVec<N, T> {
	fn borrow_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<const N: usize, T> core::ops::Deref for SwitchVec<N, T> {
	type Target = [T];
	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl<const N: usize, T> core::ops::DerefMut for SwitchVec<N, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut_slice()
	}
}

impl<const N: usize, T, I: core::slice::SliceIndex<[T]>> core::ops::Index<I> for SwitchVec<N, T> {
	type Output = I::Output;
	fn index(&self, index: I) -> &Self::Output {
		core::ops::Index::index(self.as_slice(), index)
	}
}

impl<const N: usize, T, I: core::slice::SliceIndex<[T]>> core::ops::IndexMut<I> for SwitchVec<N, T> {
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		core::ops::IndexMut::index_mut(self.as_mut_slice(), index)
	}
}

impl<const N: usize, T> Extend<T> for SwitchVec<N, T> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		for i in iter {
			if self.push(i).is_err() {
				break;
			}
		}
	}
}

impl<'a, const N: usize, T: Copy> Extend<&'a T> for SwitchVec<N, T> {
	fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
		for i in iter {
			if self.push(*i).is_err() {
				break;
			}
		}
	}
}


enum IntoIterInner<const N: usize, T> {
	Stack(crate::array::IntoIter<N, T>),
	#[cfg(feature = "std")]
	Heap(std::vec::IntoIter<T>),
}

/// iterator for [`SwitchVec`].
pub struct IntoIter<const N: usize, T> {
	inner: IntoIterInner<N, T>,
}

impl<const N: usize, T> Iterator for IntoIter<N, T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IntoIterInner::Stack(array) => array.next(),
			#[cfg(feature = "std")]
			IntoIterInner::Heap(vec) => vec.next(),
		}
	}
}

impl<const N: usize, T> DoubleEndedIterator for IntoIter<N, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IntoIterInner::Stack(array) => array.next_back(),
			#[cfg(feature = "std")]
			IntoIterInner::Heap(vec) => vec.next_back(),
		}
	}
}

impl<const N: usize, T> IntoIterator for SwitchVec<N, T> {
	type IntoIter = IntoIter<N, T>;
	type Item = T;
	
	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: match self.inner {
				Inner::Stack(array) => IntoIterInner::Stack(array.into_iter()),
				#[cfg(feature = "std")]
				Inner::Heap(vec) => IntoIterInner::Heap(vec.into_iter()),
			},
		}
	}
}

impl<'a, const N: usize, T> IntoIterator for &'a SwitchVec<N, T> {
	type IntoIter = core::slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.as_slice().iter()
	}
}

impl<'a, const N: usize, T> IntoIterator for &'a mut SwitchVec<N, T> {
	type IntoIter = core::slice::IterMut<'a, T>;
	type Item = &'a mut T;

	fn into_iter(self) -> Self::IntoIter {
		self.as_mut_slice().iter_mut()
	}
}

impl<const N: usize, T> FromIterator<T> for SwitchVec<N, T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut out = Self::new();
		out.extend(iter);
		out
	}
}


impl<const N: usize, T: PartialOrd> PartialOrd for SwitchVec<N, T> {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, T: Eq> Eq for SwitchVec<N, T> {}

impl<const N: usize, T: Ord> Ord for SwitchVec<N, T> {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		Ord::cmp(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, const M: usize, T: PartialEq> PartialEq<SwitchVec<M, T>> for SwitchVec<N, T> {
	fn eq(&self, other: &SwitchVec<M, T>) -> bool {
		PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, T: PartialEq> PartialEq<&[T]> for SwitchVec<N, T> {
	fn eq(&self, other: &&[T]) -> bool {
		PartialEq::eq(self.as_slice(), *other)
	}
}

impl<const N: usize, T: PartialEq> PartialEq<&mut [T]> for SwitchVec<N, T> {
	fn eq(&self, other: &&mut [T]) -> bool {
		PartialEq::eq(self.as_slice(), *other)
	}
}

impl<const N: usize, const M: usize, T: PartialEq> PartialEq<[T; M]> for SwitchVec<N, T> {
	fn eq(&self, other: &[T; M]) -> bool {
		PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, const M: usize, T: PartialEq> PartialEq<&[T; M]> for SwitchVec<N, T> {
	fn eq(&self, other: &&[T; M]) -> bool {
		PartialEq::eq(self.as_slice(), other.as_slice())
	}
}

impl<const N: usize, T: core::fmt::Debug> core::fmt::Debug for SwitchVec<N, T> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		core::fmt::Debug::fmt(self.as_slice(), f)
	}
}

