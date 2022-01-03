use std::collections::HashSet;

use tokio::sync::mpsc::Receiver;

use crate::h264::H264NalUnit;
use crate::utils::SocksContainer;

const FRAME_BUFFER_CAP: usize = 1000;

pub async fn write_h264_stream(mut producer: Receiver<H264NalUnit>, socks: SocksContainer) {
	let mut seq_param: Option<H264NalUnit> = None;
	let mut pic_param: Option<H264NalUnit> = None;

	let mut frame_buffer = Vec::with_capacity(FRAME_BUFFER_CAP);
	let mut known_socks = HashSet::new();

	while let Some(frame) = producer.recv().await {
		match frame.unit_code {
			7 => {
				seq_param = Some(frame);
				continue;
			}
			8 => {
				pic_param = Some(frame);
				continue;
			}
			5 => frame_buffer.clear(),
			_ => {}
		}

		let mut errors = Vec::with_capacity(0);

		{
			for (addr, sock) in socks.lock().await.iter_mut() {
				let write = if !known_socks.contains(addr) {
					known_socks.insert(addr.to_owned());
					let mut frames = [&seq_param, &pic_param]
						.into_iter()
						.filter(|p| p.is_some())
						.flat_map(|p| p.into_iter())
						.collect::<Vec<_>>();
					frames.extend(&frame_buffer);
					frames.push(&frame);

					let mut buffer = Vec::with_capacity(
						frames.iter()
							.map(|x| x.raw_bytes.len())
							.fold(0, |x, y| x + y)
					);

					for frame in frames {
						buffer.extend(&frame.raw_bytes);
					}

					sock.write_all(&buffer).await
				} else {
					sock.write_all(&frame.raw_bytes).await
				};

				if let Err(_) = write {
					errors.push(addr.to_owned());
				}
			}
		}

		while let Some(addr) = errors.pop() {
			socks.lock().await.remove(&addr);
			info!("Ejected {}", addr);
		}

		// Drop the buffer if it's too big
		if frame_buffer.len() >= FRAME_BUFFER_CAP {
			frame_buffer.truncate(1);
		}

		frame_buffer.push(frame);
	}
}
