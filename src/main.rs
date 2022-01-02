#[macro_use]
extern crate argh;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::process;
use std::sync::Arc;

use tokio::io;
use tokio::io::stdin;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::cli::CliArgs;
use crate::h264::{H264NalUnit, H264Stream};

mod cli;
mod h264;

pub type Am<T> = Arc<Mutex<T>>;

pub fn am<T>(t: T) -> Am<T> {
	Arc::new(Mutex::new(t))
}

#[tokio::main]
async fn main() {
	let args = argh::from_env::<CliArgs>();

	let listener = args.start_listening().await;
}

async fn read_h264_stream(consumer: Sender<H264NalUnit>) {
	let mut stream = H264Stream::new(stdin());

	while let Ok(frame) = stream.next().await {
		let _ = consumer.send(frame).await;
	}
}

async fn listen_for_new_sockets(listener: TcpListener, socks: Am<Vec<Am<TcpStream>>>) {
	while let Ok((client, addr)) = listener.accept().await {
		let client = am(client);
		// Register socket with the reader thread
		{
			socks.lock().await.push(client.clone());
		}
	}
}