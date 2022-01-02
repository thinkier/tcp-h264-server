#[macro_use]
extern crate argh;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate tokio;

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;

use log::LevelFilter;
use tokio::io::{AsyncWriteExt, stdin};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::cli::CliArgs;
use crate::h264::{H264NalUnit, H264Stream};

mod cli;
mod h264;

pub type Am<T> = Arc<Mutex<T>>;
pub type SocksContainer = Am<HashMap<SocketAddr, TcpStream>>;

pub fn am<T>(t: T) -> Am<T> {
	Arc::new(Mutex::new(t))
}

#[tokio::main]
async fn main() {
	let args = argh::from_env::<CliArgs>();

	env_logger::builder()
		.filter_level(LevelFilter::Info)
		.init();

	let (tx, rx) = mpsc::channel(60);
	let socks = am(HashMap::new());
	tokio::spawn(read_h264_stream(tx));
	tokio::spawn(write_h264_stream(rx, socks.clone()));

	let listener = args.start_listening().await;
	listen_for_new_sockets(listener, socks).await;
}

async fn read_h264_stream(consumer: Sender<H264NalUnit>) {
	let mut stream = H264Stream::new(stdin());

	// TODO Spawn a raspivid / libcamera-vid process
	info!("Capturing H.264 video from /dev/stdin");

	while let Ok(frame) = stream.next().await {
		let _ = consumer.send(frame).await;
	}
}

const FRAME_BUFFER_CAP: usize = 1000;

async fn write_h264_stream(mut producer: Receiver<H264NalUnit>, socks: SocksContainer) {
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
					let parameters = [&seq_param, &pic_param]
						.into_iter()
						.filter(|p| p.is_some())
						.map(Clone::clone)
						.map(Option::unwrap)
						.collect::<Vec<_>>();

					let mut buffer = Vec::with_capacity(
						parameters.iter().fold(0, count_nal_bytes) +
							frame_buffer.iter().fold(0, count_nal_bytes) +
							frame.raw_bytes.len()
					);

					for nal in &parameters {
						buffer.extend(&nal.raw_bytes);
					}
					for nal in &frame_buffer {
						buffer.extend(&nal.raw_bytes);
					}
					buffer.extend(&frame.raw_bytes);

					sock.write_all(&buffer).await
				} else {
					sock.write_all(&frame.raw_bytes).await
				};

				if let Err(e) = write {
					errors.push(addr.to_owned());
					warn!("Write error {} on {}, disconnecting...",e.description(), addr);
				}
			}
		}

		while let Some(addr) = errors.pop() {
			socks.lock().await.remove(&addr);
		}

		// Drop the buffer if it's too big
		if frame_buffer.len() >= FRAME_BUFFER_CAP {
			frame_buffer.truncate(1);
		}

		frame_buffer.push(frame);
	}
}

fn count_nal_bytes(x: usize, nal: &H264NalUnit) -> usize {
	x + nal.raw_bytes.len()
}

async fn listen_for_new_sockets(listener: TcpListener, socks: SocksContainer) {
	while let Ok((client, addr)) = listener.accept().await {
		info!("Incoming connection at {:?}",addr);
		socks.lock().await.insert(addr, client);
	}
}
