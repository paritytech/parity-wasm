use std::u32;
use std::sync::Arc;
use parking_lot::RwLock;
use elements::MemoryType;
use interpreter::Error;
use interpreter::module::check_limits;

/// Linear memory page size.
pub const LINEAR_MEMORY_PAGE_SIZE: u32 = 65536;
/// Maximal number of pages.
const LINEAR_MEMORY_MAX_PAGES: u32 = 65536;

/// Linear memory instance.
pub struct MemoryInstance {
	/// Linear memory buffer.
	buffer: RwLock<Vec<u8>>,
	/// Maximum buffer size.
	maximum_size: u32,
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

impl MemoryInstance {
	/// Create new linear memory instance.
	pub fn new(memory_type: &MemoryType) -> Result<Arc<Self>, Error> {
		check_limits(memory_type.limits())?;

		if let Some(maximum_pages) = memory_type.limits().maximum() {
			if maximum_pages > LINEAR_MEMORY_MAX_PAGES {
				return Err(Error::Memory(format!("memory size must be at most 65536 pages")));
			}
		}

		let memory = MemoryInstance {
			buffer: RwLock::new(Vec::new()), // TODO: with_capacity
			maximum_size: memory_type.limits().maximum()
				.map(|s| s.saturating_mul(LINEAR_MEMORY_PAGE_SIZE))
				.unwrap_or(u32::MAX),
		};
		if memory.grow(memory_type.limits().initial())? == u32::MAX {
			return Err(Error::Memory(format!("error initializing {}-pages linear memory region", memory_type.limits().initial())));
		}
		Ok(Arc::new(memory))
	}

	/// Return linear memory size (in pages).
	pub fn size(&self) -> u32 {
		self.buffer.read().len() as u32 / LINEAR_MEMORY_PAGE_SIZE
	}

	/// Get data at given offset.
	pub fn get(&self, offset: u32, size: usize) -> Result<Vec<u8>, Error> {
		let buffer = self.buffer.read();
		let region = self.checked_region(&buffer, offset as usize, size)?;

		Ok(region.slice().to_vec())
	}

	/// Set data at given offset.
	pub fn set(&self, offset: u32, value: &[u8]) -> Result<(), Error> {
		let mut buffer = self.buffer.write();
		let range = self.checked_region(&buffer, offset as usize, value.len())?.range();

		buffer[range].copy_from_slice(value);

		Ok(())
	}

	/// Increases the size of the linear memory by given number of pages.
	/// Returns -1 if allocation fails or previous memory size, if succeeds.
	pub fn grow(&self, pages: u32) -> Result<u32, Error> {
		let mut buffer = self.buffer.write();
		let old_size = buffer.len() as u32;
		match pages.checked_mul(LINEAR_MEMORY_PAGE_SIZE).and_then(|bytes| old_size.checked_add(bytes)) {
			None => Ok(u32::MAX),
			Some(new_size) if new_size > self.maximum_size => Ok(u32::MAX),
			Some(new_size) => {
				buffer.extend(vec![0; (new_size - old_size) as usize]);
				Ok(old_size / LINEAR_MEMORY_PAGE_SIZE)
			},
		}
	}

	fn checked_region<'a, B>(&self, buffer: &'a B, offset: usize, size: usize) -> Result<CheckedRegion<'a, B>, Error> 
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
	pub fn copy(&self, src_offset: usize, dst_offset: usize, len: usize) -> Result<(), Error> {
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
	pub fn zero(&self, offset: usize, len: usize) -> Result<(), Error> {
		let mut buffer = self.buffer.write();

		let range = self.checked_region(&buffer, offset, len)?.range();
		for val in &mut buffer[range] { *val = 0 }
		Ok(())
	}
}
