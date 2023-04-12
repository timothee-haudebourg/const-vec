//! This library provides a `Vec`-like data structure called `ConstVec` where
//! elements can be pushed to the array in an immutable way as long as the
//! capacity of the vector is large enough).
//!
//! # Example
//!
//! ```rust
//! use const_vec::ConstVec;
//!
//! // Create a new empty `ConstVec` with a capacity of 10 items.
//! // Note that it is NOT mutable.
//! let vec = ConstVec::new(10);
//!
//! // Add a new element in `vec`, without mutating it.
//! vec.push(42);
//! ```
use std::{
	alloc,
	alloc::Layout,
	borrow::{Borrow, BorrowMut},
	cell::Cell,
	fmt,
	mem::{self, ManuallyDrop},
	ops::{Deref, DerefMut},
	ptr,
	ptr::NonNull,
};

/// Fixed capacity array with immutable `push` method.
pub struct ConstVec<T> {
	ptr: NonNull<T>,
	capacity: usize,
	len: Cell<usize>,
}

impl<T> ConstVec<T> {
	/// Creates a new array with the given fixed capacity.
	pub fn new(capacity: usize) -> ConstVec<T> {
		let ptr = if capacity == 0 {
			NonNull::dangling()
		} else {
			let layout = Layout::array::<T>(capacity).unwrap();
			let ptr = unsafe { alloc::alloc(layout) };
			match NonNull::new(ptr as *mut T) {
				Some(ptr) => ptr,
				None => alloc::handle_alloc_error(layout),
			}
		};

		ConstVec {
			ptr,
			capacity,
			len: Cell::new(0),
		}
	}

	/// Creates a `ConstVec<T>` directly from a pointer, a capacity, and a
	/// length.
	///
	/// # Safety
	///
	/// This is highly unsafe, due to the number of invariants that aren't
	/// checked:
	///
	/// * `T` needs to have the same alignment as what `ptr` was allocated with.
	///   (`T` having a less strict alignment is not sufficient, the alignment really
	///   needs to be equal to satisfy the [`dealloc`] requirement that memory must be
	///   allocated and deallocated with the same layout.)
	/// * The size of `T` times the `capacity` (ie. the allocated size in bytes) needs
	///   to be the same size as the pointer was allocated with. (Because similar to
	///   alignment, [`dealloc`] must be called with the same layout `size`.)
	/// * `len` needs to be less than or equal to `capacity`.
	/// * The first `len` values must be properly initialized values of type `T`.
	/// * `capacity` needs to be the capacity that the pointer was allocated with.
	/// * The allocated size in bytes must be no larger than `isize::MAX`.
	///   See the safety documentation of `pointer::offset`.
	///
	/// These requirements are always upheld by any `ptr` that has been allocated
	/// via `Vec<T>`. Other allocation sources are allowed if the invariants are
	/// upheld.
	///
	/// The ownership of `ptr` is effectively transferred to the
	/// `ConstVec<T>` which may then deallocate, reallocate or change the
	/// contents of memory pointed to by the pointer at will. Ensure
	/// that nothing else uses the pointer after calling this
	/// function.
	///
	/// [`dealloc`]: alloc::dealloc
	#[inline]
	pub unsafe fn from_raw_parts(ptr: *mut T, len: usize, capacity: usize) -> Self {
		Self {
			ptr: NonNull::new_unchecked(ptr),
			len: Cell::new(len),
			capacity,
		}
	}

	/// Decomposes a `ConstVec<T>` into its raw components.
	///
	/// Returns the raw pointer to the underlying data, the length of
	/// the vector (in elements), and the allocated capacity of the
	/// data (in elements). These are the same arguments in the same
	/// order as the arguments to [`from_raw_parts`].
	///
	/// After calling this function, the caller is responsible for the
	/// memory previously managed by the `Vec`. The only way to do
	/// this is to convert the raw pointer, length, and capacity back
	/// into a `ConstVec` with the [`from_raw_parts`] function, allowing
	/// the destructor to perform the cleanup.
	///
	/// [`from_raw_parts`]: ConstVec::from_raw_parts
	///
	/// # Examples
	///
	/// ```
	/// # use const_vec::ConstVec;
	/// let v: ConstVec<i32> = ConstVec::new(3);
	/// v.push(-1);
	/// v.push(0);
	/// v.push(1);
	///
	/// let (ptr, len, cap) = v.into_raw_parts();
	///
	/// let rebuilt = unsafe {
	///     // We can now make changes to the components, such as
	///     // transmuting the raw pointer to a compatible type.
	///     let ptr = ptr as *mut u32;
	///
	///     ConstVec::from_raw_parts(ptr, len, cap)
	/// };
	///
	/// assert_eq!(rebuilt, [4294967295, 0, 1]);
	/// ```
	pub fn into_raw_parts(self) -> (*mut T, usize, usize) {
		let mut me = ManuallyDrop::new(self);
		(me.as_mut_ptr(), me.len(), me.capacity())
	}

	#[inline]
	pub fn capacity(&self) -> usize {
		self.capacity
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.len.get()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	#[inline]
	pub fn as_ptr(&self) -> *const T {
		self.ptr.as_ptr()
	}

	#[inline]
	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.ptr.as_ptr()
	}

	#[inline]
	pub fn as_slice(&self) -> &[T] {
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	#[inline]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
	}

	#[inline]
	pub fn push(&self, value: T) {
		if self.len() < self.capacity() {
			unsafe {
				let len = self.len.get();
				let end = self.ptr.as_ptr().add(len);
				std::ptr::write(end, value);
				self.len.set(len + 1);
			}
		} else {
			panic!("not enough capacity")
		}
	}

	/// Removes the last element from a vector and returns it, or [`None`] if it
	/// is empty.
	///
	/// # Examples
	///
	/// ```
	/// # use const_vec::ConstVec;
	/// let mut vec = ConstVec::new(3);
	/// vec.push(1);
	/// vec.push(2);
	/// vec.push(3);
	///
	/// assert_eq!(vec.pop(), Some(3));
	/// assert_eq!(vec, [1, 2]);
	/// ```
	#[inline]
	pub fn pop(&mut self) -> Option<T> {
		if self.len.get() == 0 {
			None
		} else {
			unsafe {
				self.len.set(self.len.get() - 1);
				Some(ptr::read(self.as_ptr().add(self.len())))
			}
		}
	}

	/// Moves all the elements of `other` into `self`, leaving `other` empty.
	///
	/// # Panics
	///
	/// Panics if the current length and `other` length exceed the capacity.
	///
	/// # Examples
	///
	/// ```
	/// # use const_vec::ConstVec;
	/// let vec = ConstVec::new(6);
	/// vec.push(1);
	/// vec.push(2);
	/// vec.push(3);
	///
	/// let mut vec2 = vec![4, 5, 6];
	/// vec.append(&mut vec2);
	///
	/// assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
	/// assert_eq!(vec2, []);
	/// ```
	pub fn append(&self, other: &mut Vec<T>) {
		if self.len() + other.len() <= self.capacity() {
			unsafe {
				self.append_elements(other.as_slice() as _);
				other.set_len(0)
			}
		} else {
			panic!("not enough capacity")
		}
	}

	/// Appends elements to `self` from other buffer.
	///
	/// The sum of the current length and length of `other` must not exceed
	/// the capacity.
	#[cfg(not(no_global_oom_handling))]
	#[inline]
	unsafe fn append_elements(&self, other: *const [T]) {
		let count = unsafe { (*other).len() };
		let len = self.len();
		unsafe { ptr::copy_nonoverlapping(other as *const T, self.ptr.as_ptr().add(len), count) };
		self.len.set(len + count);
	}

	/// Clears the vector, removing all values.
	///
	/// Note that this method has no effect on the allocated capacity
	/// of the vector.
	///
	/// # Examples
	///
	/// ```
	/// # use const_vec::ConstVec;
	/// let mut vec = ConstVec::new(3);
	/// vec.push(1);
	/// vec.push(2);
	/// vec.push(3);
	///
	/// vec.clear();
	///
	/// assert!(vec.is_empty());
	/// ```
	#[inline]
	pub fn clear(&mut self) {
		let elems: *mut [T] = self.as_mut_slice();

		// SAFETY:
		// - `elems` comes directly from `as_mut_slice` and is therefore valid.
		// - Setting `self.len` before calling `drop_in_place` means that,
		//   if an element's `Drop` impl panics, the vector's `Drop` impl will
		//   do nothing (leaking the rest of the elements) instead of dropping
		//   some twice.
		unsafe {
			self.len.set(0);
			ptr::drop_in_place(elems);
		}
	}
}

impl<T> IntoIterator for ConstVec<T> {
	type IntoIter = IntoIter<T>;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		let iter = IntoIter {
			ptr: self.ptr,
			capacity: self.capacity,
			start: self.ptr.as_ptr(),
			end: unsafe { self.ptr.as_ptr().add(self.len()) },
		};

		mem::forget(self);
		iter
	}
}

impl<'a, T> IntoIterator for &'a ConstVec<T> {
	type IntoIter = std::slice::Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<T: Clone> Clone for ConstVec<T> {
	fn clone(&self) -> Self {
		let result = Self::new(self.capacity);

		for item in self {
			result.push(item.clone())
		}

		result
	}
}

impl<T> AsRef<[T]> for ConstVec<T> {
	fn as_ref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<T> AsMut<[T]> for ConstVec<T> {
	fn as_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<T> Borrow<[T]> for ConstVec<T> {
	fn borrow(&self) -> &[T] {
		self.as_slice()
	}
}

impl<T> BorrowMut<[T]> for ConstVec<T> {
	fn borrow_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<T> Deref for ConstVec<T> {
	type Target = [T];

	#[inline]
	fn deref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<T> DerefMut for ConstVec<T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

impl<T> Drop for ConstVec<T> {
	fn drop(&mut self) {
		if self.capacity != 0 {
			unsafe {
				// use drop for [T]
				// use a raw slice to refer to the elements of the vector as weakest necessary type;
				// could avoid questions of validity in certain cases
				ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len()));

				let layout = Layout::array::<T>(self.capacity).unwrap();
				alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
			}
		}
	}
}

impl<T: fmt::Debug> fmt::Debug for ConstVec<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&**self, f)
	}
}

impl<T: PartialEq<U>, U> PartialEq<[U]> for ConstVec<T> {
	#[inline]
	fn eq(&self, other: &[U]) -> bool {
		*self.as_slice() == *other
	}
}

impl<'a, T: PartialEq<U>, U> PartialEq<&'a [U]> for ConstVec<T> {
	#[inline]
	fn eq(&self, other: &&'a [U]) -> bool {
		*self.as_slice() == **other
	}
}

impl<T: PartialEq<U>, U, const N: usize> PartialEq<[U; N]> for ConstVec<T> {
	#[inline]
	fn eq(&self, other: &[U; N]) -> bool {
		*self.as_slice() == *other
	}
}

impl<'a, T: PartialEq<U>, U, const N: usize> PartialEq<&'a [U; N]> for ConstVec<T> {
	#[inline]
	fn eq(&self, other: &&'a [U; N]) -> bool {
		*self.as_slice() == **other
	}
}

impl<T: PartialEq<U>, U> PartialEq<ConstVec<U>> for ConstVec<T> {
	#[inline]
	fn eq(&self, other: &ConstVec<U>) -> bool {
		*self.as_slice() == *other.as_slice()
	}
}

impl<T> From<Vec<T>> for ConstVec<T> {
	fn from(value: Vec<T>) -> Self {
		let mut value = ManuallyDrop::new(value);
		let ptr = value.as_mut_ptr();
		let len = value.len();
		let capacity = value.capacity();
		unsafe { Self::from_raw_parts(ptr, len, capacity) }
	}
}

impl<T> From<ConstVec<T>> for Vec<T> {
	fn from(value: ConstVec<T>) -> Self {
		let (ptr, len, capacity) = value.into_raw_parts();
		unsafe { Vec::from_raw_parts(ptr, len, capacity) }
	}
}

pub struct IntoIter<T> {
	ptr: NonNull<T>,
	capacity: usize,
	start: *mut T,
	end: *mut T,
}

impl<T> IntoIter<T> {
	#[inline]
	pub fn len(&self) -> usize {
		(self.end as usize - self.start as usize) / mem::size_of::<T>()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.start == self.end
	}

	#[inline]
	pub fn as_ptr(&self) -> *const T {
		self.start
	}

	#[inline]
	pub fn as_mut_ptr(&mut self) -> *mut T {
		self.start
	}

	#[inline]
	pub fn as_slice(&self) -> &[T] {
		unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len()) }
	}

	#[inline]
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe { std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
	}
}

impl<T> Iterator for IntoIter<T> {
	type Item = T;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.len();
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		if self.start == self.end {
			None
		} else {
			unsafe {
				let result = ptr::read(self.start);
				self.start = self.start.offset(1);
				Some(result)
			}
		}
	}
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> DoubleEndedIterator for IntoIter<T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.start == self.end {
			None
		} else {
			unsafe {
				self.end = self.end.offset(-1);
				Some(ptr::read(self.end))
			}
		}
	}
}

impl<T> Drop for IntoIter<T> {
	fn drop(&mut self) {
		if self.capacity != 0 {
			unsafe {
				// use drop for [T]
				// use a raw slice to refer to the elements of the vector as weakest necessary type;
				// could avoid questions of validity in certain cases
				ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len()));

				let layout = Layout::array::<T>(self.capacity).unwrap();
				alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
			}
		}
	}
}
