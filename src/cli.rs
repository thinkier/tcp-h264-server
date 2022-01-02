use std::net::SocketAddrV4;

use tokio::net::TcpListener;

#[derive(FromArgs)]
/// serve multiple H.264 streams over TCP
pub struct CliArgs {
	#[argh(option, short = 'h', default = "String::from(\"0.0.0.0\")")]
	/// the host ip (ipv4) to bind to (default: 0.0.0.0)
	pub host: String,
	#[argh(option, short = 'p', default = "1264")]
	/// the port to bind to (default: 1264)
	pub port: u16,
}

impl CliArgs {
	pub async fn start_listening(&self) -> TcpListener {
		TcpListener::bind(SocketAddrV4::new(
			self.host.parse().unwrap(),
			self.port,
		)).await.unwrap()
	}
}
