use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncWriteExt, Result as IoResult};
use tokio::net::TcpStream;
use tokio::process::ChildStdin;
use tokio::sync::Mutex;

pub type Am<T> = Arc<Mutex<T>>;
pub type StreamsContainer = Am<HashMap<String, Writable>>;

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
