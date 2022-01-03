use std::mem;

use tokio::io::AsyncReadExt;
use tokio::io::Result as IoResult;

const NAL_UNIT_PREFIX_NULL_BYTES: usize = 2;

pub struct H264Stream<R> {
	reader: R,
	buffer: Vec<u8>,
	nal_unit_detect: usize,
}

impl<R: AsyncReadExt + Unpin> H264Stream<R> {
	pub fn new(reader: R) -> Self {
		H264Stream {
			reader,
			// initial 4MiB buffer
			buffer: Vec::with_capacity(4 << 20),
			nal_unit_detect: 0,
		}
	}

	/// Store-and-forward implementation of parsing H264 NAL Units
	pub async fn next(&mut self) -> IoResult<H264NalUnit> {
		loop {
			let read = self.reader.read_buf(&mut self.buffer).await?;
			let end = self.buffer.len();
			let start = end - read;
			for i in start..end {
				// H264 NAL Unit Header is 0x000001 https://stackoverflow.com/a/2861340/8835688
				if self.buffer[i] == 0x00 {
					self.nal_unit_detect += 1;
					continue;
				}

				// Some encoder implementations write more than 2 null bytes
				let is_nal_unit = self.nal_unit_detect >= NAL_UNIT_PREFIX_NULL_BYTES && self.buffer[i] == 0x01;
				let nal_unit_detect = mem::replace(&mut self.nal_unit_detect, 0);

				if is_nal_unit {
					let last_frame_end = i - nal_unit_detect;
					// If we're at the start of the h264 stream there's no previous unit to emit
					if last_frame_end == 0 {
						continue;
					}

					// Extract NAL unit
					let last_frame_start = nal_unit_detect - NAL_UNIT_PREFIX_NULL_BYTES;
					let mut nal_unit = Vec::with_capacity(last_frame_end);
					nal_unit.extend(&self.buffer[last_frame_start..last_frame_end]);


					// Move to the start (with allocation)
					{
						let mut buffered = Vec::with_capacity(end - last_frame_end);
						buffered.extend(&self.buffer[last_frame_end..]);
						self.buffer.clear();
						self.buffer.extend(&buffered);
					}

					return Ok(H264NalUnit::new(nal_unit));
				}
			}
		}
	}
}

#[derive(Clone)]
pub struct H264NalUnit {
	pub unit_code: u8,
	pub raw_bytes: Vec<u8>,
}

impl H264NalUnit {
	pub fn new(raw_bytes: Vec<u8>) -> Self {
		H264NalUnit {
			// There's 32 possible NAL unit codes for H264
			unit_code: 0x1f & raw_bytes[3],
			raw_bytes,
		}
	}
}
