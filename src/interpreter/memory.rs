use std::u32;
use std::sync::Arc;
use parking_lot::RwLock;
use elements::{MemoryType, ResizableLimits};
use interpreter::{Error, UserError};
use interpreter::module::check_limits;

/// Linear memory page size.
pub const LINEAR_MEMORY_PAGE_SIZE: u32 = 65536;
/// Maximal number of pages.
const LINEAR_MEMORY_MAX_PAGES: u32 = 65536;

/// Linear memory instance.
pub struct MemoryInstance<E: UserError> {
	/// Memofy limits.
	limits: ResizableLimits,
	/// Linear memory buffer.
	buffer: RwLock<Vec<u8>>,
	/// Maximum buffer size.
	maximum_size: u32,
	/// Dummy to avoid compilation error.
	_dummy: ::std::marker::PhantomData<E>,
}

struct CheckedRegion<'a, B: 'a> where B: ::std::ops::Deref<Target=Vec<u8>> {
	buffer: &'a B,
	offset: usize,
	size: usize,
}

impl<'a, B: 'a> CheckedRegion<'a, B> where B: ::std::ops::Deref<Target=Vec<u8>> {
	fn range(&self) -> ::std::ops::Range<usize> {
		self.offset..self.offset+self.size
	}

	fn slice(&self) -> &[u8] {
		&self.buffer[self.range()]
	}
}

impl<E> MemoryInstance<E> where E: UserError {
	/// Create new linear memory instance.
	pub fn new(memory_type: &MemoryType) -> Result<Arc<Self>, Error<E>> {
		check_limits(memory_type.limits())?;

		let maximum_size = match memory_type.limits().maximum() {
			Some(maximum_pages) if maximum_pages > LINEAR_MEMORY_MAX_PAGES =>
				return Err(Error::Memory(format!("maximum memory size must be at most {} pages", LINEAR_MEMORY_MAX_PAGES))),
			Some(maximum_pages) => maximum_pages.saturating_mul(LINEAR_MEMORY_PAGE_SIZE),
			None => u32::MAX,
		};
		let initial_size = calculate_memory_size(0, memory_type.limits().initial(), maximum_size)
			.ok_or(Error::Memory(format!("initial memory size must be at most {} pages", LINEAR_MEMORY_MAX_PAGES)))?;

		let memory = MemoryInstance {
			limits: memory_type.limits().clone(),
			buffer: RwLock::new(vec![0; initial_size as usize]),
			maximum_size: maximum_size,
			_dummy: Default::default(),
		};

		Ok(Arc::new(memory))
	}

	/// Return linear memory limits.
	pub fn limits(&self) -> &ResizableLimits {
		&self.limits
	}

	/// Return linear memory size (in pages).
	pub fn size(&self) -> u32 {
		self.buffer.read().len() as u32 / LINEAR_MEMORY_PAGE_SIZE
	}

	/// Get data at given offset.
	pub fn get(&self, offset: u32, size: usize) -> Result<Vec<u8>, Error<E>> {
		let buffer = self.buffer.read();
		let region = self.checked_region(&buffer, offset as usize, size)?;

		Ok(region.slice().to_vec())
	}

	/// Set data at given offset.
	pub fn set(&self, offset: u32, value: &[u8]) -> Result<(), Error<E>> {
		let mut buffer = self.buffer.write();
		let range = self.checked_region(&buffer, offset as usize, value.len())?.range();

		buffer[range].copy_from_slice(value);

		Ok(())
	}

	/// Increases the size of the linear memory by given number of pages.
	/// Returns -1 if allocation fails or previous memory size, if succeeds.
	pub fn grow(&self, pages: u32) -> Result<u32, Error<E>> {
		let mut buffer = self.buffer.write();
		let old_size = buffer.len() as u32;
		match calculate_memory_size(old_size, pages, self.maximum_size) {
			None => Ok(u32::MAX),
			Some(new_size) => {
				buffer.resize(new_size as usize, 0);
				Ok(old_size / LINEAR_MEMORY_PAGE_SIZE)
			},
		}
	}

	fn checked_region<'a, B>(&self, buffer: &'a B, offset: usize, size: usize) -> Result<CheckedRegion<'a, B>, Error<E>> 
		where B: ::std::ops::Deref<Target=Vec<u8>>
	{
		let end = offset.checked_add(size)
			.ok_or(Error::Memory(format!("trying to access memory block of size {} from offset {}", size, offset)))?;

		if end > buffer.len() {
			return Err(Error::Memory(format!("trying to access region [{}..{}] in memory [0..{}]", offset, end, buffer.len())));
		}

		Ok(CheckedRegion {
			buffer: buffer,
			offset: offset,
			size: size,
		})
	}

	/// Copy memory region
	pub fn copy(&self, src_offset: usize, dst_offset: usize, len: usize) -> Result<(), Error<E>> {
		let buffer = self.buffer.write();

		let read_region = self.checked_region(&buffer, src_offset, len)?;
		let write_region = self.checked_region(&buffer, dst_offset, len)?;

		unsafe { ::std::ptr::copy(
			buffer[read_region.range()].as_ptr(), 
			buffer[write_region.range()].as_ptr() as *mut _,
			len,
		)} 

		Ok(())
	}

	/// Zero memory region
	pub fn zero(&self, offset: usize, len: usize) -> Result<(), Error<E>> {
		let mut buffer = self.buffer.write();

		let range = self.checked_region(&buffer, offset, len)?.range();
		for val in &mut buffer[range] { *val = 0 }
		Ok(())
	}
}

fn calculate_memory_size(old_size: u32, additional_pages: u32, maximum_size: u32) -> Option<u32> {
	additional_pages
		.checked_mul(LINEAR_MEMORY_PAGE_SIZE)
		.and_then(|size| size.checked_add(old_size))
		.and_then(|size| if size > maximum_size {
			None
		} else {
			Some(size)
		})
}
