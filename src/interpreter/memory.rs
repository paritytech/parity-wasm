use std::u32;
use std::sync::Arc;
use parking_lot::RwLock;
use elements::MemoryType;
use interpreter::Error;

/// Linear memory page size.
pub const LINEAR_MEMORY_PAGE_SIZE: u32 = 65536;

/// Linear memory instance.
pub struct MemoryInstance {
	/// Linear memory buffer.
	buffer: RwLock<Vec<u8>>,
	/// Maximum buffer size.
	maximum_size: u32,
}

impl MemoryInstance {
	/// Create new linear memory instance.
	pub fn new(memory_type: &MemoryType) -> Result<Arc<Self>, Error> {
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
		let begin = offset as usize;
		let end = match begin.checked_add(size) {
			Some(end) => end,
			None => return Err(Error::Memory(format!("trying to read memory block of size {} from offset {}", size, offset))),
		};

		let buffer = self.buffer.read();
		if buffer.len() < end {
			return Err(Error::Memory(format!("trying to read region [{}..{}] in memory [0..{}]", begin, end, buffer.len())));
		}

		Ok(buffer[begin..end].to_vec())
	}

	/// Set data at given offset.
	pub fn set(&self, offset: u32, value: &[u8]) -> Result<(), Error> {
		let size = value.len();
		let begin = offset as usize;
		let end = match begin.checked_add(size) {
			Some(end) => end,
			None => return Err(Error::Memory(format!("trying to update memory block of size {} from offset {}", size, offset))),
		};

		let mut buffer = self.buffer.write();
		if buffer.len() <= end {
			return Err(Error::Memory(format!("trying to update region [{}..{}] in memory [0..{}]", begin, end, buffer.len())));
		}

		let mut mut_buffer = buffer.as_mut_slice();
		mut_buffer[begin..end].copy_from_slice(value);

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
}
