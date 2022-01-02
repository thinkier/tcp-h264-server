#[macro_use]
extern crate argh;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate tokio;

use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use log::LevelFilter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream, Result as IoResult, stdin};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::{ChildStdin, Command};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::interval;

use crate::cli::CliArgs;
use crate::h264::{H264NalUnit, H264Stream};

mod cli;
mod h264;

pub type Am<T> = Arc<Mutex<T>>;
pub type SocksContainer = Am<HashMap<String, Writable>>;

pub enum Writable {
	TcpStream(TcpStream),
	ChildStdin(ChildStdin),
}

impl Writable {
	pub async fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
		match self {
			Writable::TcpStream(x) => x.write_all(buf).await,
			Writable::ChildStdin(x) => x.write_all(buf).await,
		}
	}
}

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

	let img = args.start_listening_for_image().await;
	tokio::spawn(listen_for_new_image_requests(img, socks.clone()));

	let vid = args.start_listening_for_video().await;
	listen_for_new_video_sockets(vid, socks).await;
}

async fn read_h264_stream(consumer: Sender<H264NalUnit>) {
	let mut stream = H264Stream::new(stdin());

	// TODO Spawn a raspivid / libcamera-vid process
	info!("Capturing H.264 video from stdin");

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
		}

		// Drop the buffer if it's too big
		if frame_buffer.len() >= FRAME_BUFFER_CAP {
			frame_buffer.truncate(1);
		}

		frame_buffer.push(frame);
	}
}

async fn listen_for_new_image_requests(listener: TcpListener, socks: SocksContainer) {
	while let Ok((mut client, addr)) = listener.accept().await {
		let socks = socks.clone();
		tokio::spawn(async move {
			let addr = addr.to_string();

			info!("Image requested {}", addr);
			let mut child = Command::new("ffmpeg")
				.args(&[
					"-i", "-",
					"-ss", "0.5",
					"-vframes", "1",
					"-f", "image2",
					"pipe:"
				])
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.stderr(Stdio::null())
				.spawn()
				.unwrap();

			if let Some(stdin) = child.stdin.take() {
				socks.lock().await.insert(addr.clone(), Writable::ChildStdin(stdin));
			} else {
				error!("Failed to obtain stdin of ffmpeg for {}", addr);
				emit_http_500(client).await;
				return;
			}

			if let Ok(output) = child.wait_with_output().await {
				let stdout = output.stdout;
				let _ = client.write_all(format!("HTTP/1.1 200\r\n\
				Content-Type: image/jpeg\r\n\
				Content-Length: {}\r\n\r\n", stdout.len())
					.as_bytes()).await;

				let _ = client.write_all(&stdout).await;
				return;
			} else {
				error!("Failed to read ffmpeg capture for {}", addr);
				emit_http_500(client).await;
				return;
			}
		});
	}
}

async fn emit_http_500(mut client: TcpStream) {
	let _ = client.write_all(b"HTTP/1.1 500\r\n\
				Content-Length: 0").await;
}

async fn listen_for_new_video_sockets(listener: TcpListener, socks: SocksContainer) {
	while let Ok((client, addr)) = listener.accept().await {
		let addr = addr.to_string();
		info!("Streaming to {}", addr);
		socks.lock().await.insert(addr.clone(), Writable::TcpStream(client));

		let socks = socks.clone();
		tokio::spawn(async move {
			let mut int = interval(Duration::from_millis(50));

			loop {
				int.tick().await;

				if !socks.lock().await.contains_key(&addr) {
					info!("Disconnected {}", addr);
					return;
				}
			}
		});
	}
}
