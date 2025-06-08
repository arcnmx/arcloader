use std::{char::DecodeUtf16, io::{self, BufRead, Read}, iter, slice};

pub struct WideUtf8Reader<'a> {
	pub decoder: iter::Fuse<DecodeUtf16<iter::Copied<slice::Iter<'a, u16>>>>,
	pub buf: [u8; 4],
	pub buf_pos: usize,
	pub buf_len: usize,
}

impl<'a> WideUtf8Reader<'a> {
	pub fn new(widebuf: &'a [u16]) -> Self {
		Self {
			decoder: char::decode_utf16(widebuf.iter().copied()).fuse(),
			buf: [0u8; 4],
			buf_pos: 0,
			buf_len: 0,
		}
	}
}

impl<'a> BufRead for WideUtf8Reader<'a> {
	fn fill_buf(&mut self) -> io::Result<&[u8]> {
		if self.buf_pos >= self.buf_len {
			let res = self.decoder.next().transpose()
				.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
			match res {
				Some(c) => {
					let len = c.encode_utf8(&mut self.buf).len();
					self.buf_len = len;
				},
				None => {
					self.buf_len = 0;
				},
			}
			self.buf_pos = 0;
		}
		Ok(&self.buf[self.buf_pos..self.buf_len])
	}

	fn consume(&mut self, amt: usize) {
		self.buf_pos += amt;
	}
}

impl<'a> Read for WideUtf8Reader<'a> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let src = self.fill_buf()?;
		let read_len = src.len().min(buf.len());
		buf[..read_len].copy_from_slice(&src[..read_len]);
		self.consume(read_len);
		Ok(read_len)
	}
}

impl<'a, T: AsRef<[u16]>> From<&'a T> for WideUtf8Reader<'a> {
	fn from(s: &'a T) -> Self {
		Self::new(s.as_ref())
	}
}
