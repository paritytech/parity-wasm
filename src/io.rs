//! Simple abstractions for the IO operations.
//!
//! Basically it just a replacement for the std::io that is usable from
//! the `no_std` environment.

#[cfg(feature = "std")]
use std::io;

/// IO specific error.
#[derive(Debug)]
pub enum Error {
	/// Some unexpected data left in the buffer after reading all data.
	TrailingData,

	/// Unexpected End-Of-File
	UnexpectedEof,

	/// Invalid data is encountered.
	InvalidData,

	#[cfg(feature = "std")]
	Io(std::io::Error),

	/// Invalid offset for seek
	#[cfg(feature = "offsets")]
	InvalidSeek,
}

/// IO specific Result.
pub type Result<T> = core::result::Result<T, Error>;

pub trait Write {
	/// Write a buffer of data into this write.
	///
	/// All data is written at once.
	fn write(&mut self, buf: &[u8]) -> Result<()>;
}

pub trait Read {
	/// Read a data from this read to a buffer.
	///
	/// If there is not enough data in this read then `UnexpectedEof` will be returned.
	fn read(&mut self, buf: &mut [u8]) -> Result<()>;
}

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the `Seek` trait
#[cfg(feature = "offsets")]
pub enum SeekFrom {
	/// Sets the offset to the provided number of bytes.
	Start(u64),

	/// Sets the offset to the size of this object plus the specified number of bytes.
	///
	///It is possible to seek beyond the end of an object, but it’s an error to seek before byte 0.
	End(i64),

	/// Sets the offset to the current position plus the specified number of bytes.
	///
	/// It is possible to seek beyond the end of an object, but it’s an error to seek before byte 0.
	Current(i64),
}

#[cfg(feature = "offsets")]
impl From<SeekFrom> for io::SeekFrom {
	/// Convert from our implemtation of `SeekFrom` to `std::io::SeekFrom`
	fn from(seek_from: SeekFrom) -> Self {
		match seek_from {
			SeekFrom::Start(offset) => io::SeekFrom::Start(offset),
			SeekFrom::End(offset) => io::SeekFrom::End(offset),
			SeekFrom::Current(offset) => io::SeekFrom::Current(offset),
		}
	}
}

#[cfg(feature = "offsets")]
pub trait Seek {
	/// Seek to an offset, in bytes, in a stream.
	///
	/// Check [std::io::Seek](https://doc.rust-lang.org/stable/std/io/trait.Seek.html#tymethod.seek) for any
	/// details about the behaviour.
	fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

/// If the `offsets` feature is enabled,
/// we require `Read + Seek` so that we can
/// get the current position within the buffer.
#[cfg(feature = "offsets")]
pub trait ReadSeek: Read + Seek {}

/// Only require our buffer to be `Read` if
/// `offsets` is disabled.
#[cfg(not(feature = "offsets"))]
pub trait ReadSeek: Read {}

/// Reader that saves the last position.
pub struct Cursor<T> {
	inner: T,
	pos: usize,
}

impl<T> Cursor<T> {
	pub fn new(inner: T) -> Cursor<T> {
		Cursor { inner, pos: 0 }
	}

	pub fn position(&self) -> usize {
		self.pos
	}
}

impl<T: AsRef<[u8]>> ReadSeek for Cursor<T> {}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
	fn read(&mut self, buf: &mut [u8]) -> Result<()> {
		let slice = self.inner.as_ref();
		let remainder = slice.len() - self.pos;
		let requested = buf.len();
		if requested > remainder {
			return Err(Error::UnexpectedEof)
		}
		buf.copy_from_slice(&slice[self.pos..(self.pos + requested)]);
		self.pos += requested;
		Ok(())
	}
}

#[cfg(feature = "offsets")]
pub fn seek_impl(
	buffer_size: usize,
	current_position: usize,
	seek_from: SeekFrom,
) -> Result<usize> {
	fn calculate_new_position(reference_position: usize, offset: i64) -> Result<usize> {
		// Maybe I'm overly cautious here...

		// First, check if the size fits into i64...
		let reference_position: i64 =
			reference_position.try_into().map_err(|_| Error::InvalidSeek)?;

		// Then, check if there is an overflow or not
		let new_position = reference_position.checked_add(offset).ok_or(Error::InvalidSeek)?;

		// Finally, check if the new_position is < 0, which is not allowed
		// according to the documentation of std::io::SeekFrom
		if new_position < 0 {
			return Err(Error::InvalidSeek)
		}

		// On 32-bit systems, usize might be too small for a u64 value.
		// Again, maybe I'm overly cautious here
		let new_position = new_position.try_into().map_err(|_| Error::InvalidSeek)?;

		Ok(new_position)
	}

	match seek_from {
		SeekFrom::Start(offset) => {
			// On 32-bit systems, usize might be too small for a u64 value.
			// Again, maybe I'm overly cautious here
			offset.try_into().map_err(|_| Error::InvalidSeek)
		},
		SeekFrom::End(offset) => calculate_new_position(buffer_size, offset),
		SeekFrom::Current(offset) => calculate_new_position(current_position, offset),
	}
}

#[cfg(feature = "offsets")]
impl<T: AsRef<[u8]>> Seek for Cursor<T> {
	fn seek(&mut self, seek_from: SeekFrom) -> Result<u64> {
		self.pos = seek_impl(self.inner.as_ref().len(), self.pos, seek_from)?;

		// Casting up from usize to u64 should be fine though.
		Ok(self.pos as u64)
	}
}

#[cfg(not(feature = "std"))]
impl Write for alloc::vec::Vec<u8> {
	fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.extend(buf);
		Ok(())
	}
}

#[cfg(feature = "std")]
impl<T: io::Read> Read for T {
	fn read(&mut self, buf: &mut [u8]) -> Result<()> {
		self.read_exact(buf).map_err(Error::Io)
	}
}

#[cfg(feature = "offsets")]
impl<T: io::Seek> Seek for T {
	fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
		self.seek(pos.into()).map_err(Error::Io)
	}
}

#[cfg(feature = "std")]
impl<T: io::Read + io::Seek> ReadSeek for T {}

#[cfg(feature = "std")]
impl<T: io::Write> Write for T {
	fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.write_all(buf).map_err(Error::Io)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cursor() {
		let mut cursor = Cursor::new(vec![0xFFu8, 0x7Fu8]);
		assert_eq!(cursor.position(), 0);

		let mut buf = [0u8];
		assert!(cursor.read(&mut buf[..]).is_ok());
		assert_eq!(cursor.position(), 1);
		assert_eq!(buf[0], 0xFFu8);
		assert!(cursor.read(&mut buf[..]).is_ok());
		assert_eq!(buf[0], 0x7Fu8);
		assert_eq!(cursor.position(), 2);
	}

	#[test]
	fn overflow_in_cursor() {
		let mut cursor = Cursor::new(vec![0u8]);
		let mut buf = [0, 1, 2];
		assert!(cursor.read(&mut buf[..]).is_err());
	}

	#[cfg(feature = "offsets")]
	mod instruction_offset_tests {
		use super::*;

		#[test]
		fn seek_end() {
			let mut cursor = Cursor::new(vec![0xFF; 10]);

			// Trivial checks
			assert_eq!(cursor.seek(SeekFrom::End(0)).unwrap(), 10);
			assert_eq!(cursor.seek(SeekFrom::End(-1)).unwrap(), 9);
			assert_eq!(cursor.seek(SeekFrom::End(-2)).unwrap(), 8);

			// Check that we cannot seek to a position < 0
			assert!(cursor.seek(SeekFrom::End(-11)).is_err());

			// We are at position=10, check if we can go back to 0
			assert_eq!(cursor.seek(SeekFrom::End(-10)).unwrap(), 0);

			// Seek beyond the end of the buffer is allowed according to the spec
			assert_eq!(cursor.seek(SeekFrom::End(15)).unwrap(), 25);
		}

		#[test]
		fn seek_current() {
			let mut cursor = Cursor::new(vec![0xFF; 10]);

			// Trivial checks
			assert_eq!(cursor.seek(SeekFrom::Current(0)).unwrap(), 0);
			assert_eq!(cursor.seek(SeekFrom::Current(1)).unwrap(), 1);
			assert_eq!(cursor.seek(SeekFrom::Current(1)).unwrap(), 2);

			// Check that we cannot seek to a position < 0
			assert!(cursor.seek(SeekFrom::Current(-3)).is_err());

			// We are at position=10, check if we can go back to 0
			assert_eq!(cursor.seek(SeekFrom::Current(-2)).unwrap(), 0);

			// Seek beyond the end of the buffer is allowed according to the spec
			assert_eq!(cursor.seek(SeekFrom::Current(15)).unwrap(), 15);
		}

		#[test]
		fn seek_start() {
			let mut cursor = Cursor::new(vec![0xFF; 10]);

			// Trivial checks
			assert_eq!(cursor.seek(SeekFrom::Start(0)).unwrap(), 0);
			assert_eq!(cursor.seek(SeekFrom::Start(1)).unwrap(), 1);
			assert_eq!(cursor.seek(SeekFrom::Start(2)).unwrap(), 2);

			// Seek beyond the end of the buffer is allowed according to the spec
			assert_eq!(cursor.seek(SeekFrom::Start(15)).unwrap(), 15);
		}
	}
}
