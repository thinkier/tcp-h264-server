use std::net::SocketAddrV4;

use tokio::net::TcpListener;

#[derive(FromArgs)]
/// serve multiple H.264 streams over TCP
pub struct CliArgs {
	#[argh(option, short = 'h', default = "String::from(\"0.0.0.0\")")]
	/// the video host ip (ipv4) to bind to (default: 0.0.0.0)
	pub video_host: String,
	#[argh(option, short = 'p', default = "1264")]
	/// the video port to bind to (default: 1264)
	pub video_port: u16,
	#[argh(option, default = "String::from(\"0.0.0.0\")")]
	/// the image host ip (ipv4) to bind to (default: 0.0.0.0)
	pub image_host: String,
	#[argh(option, default = "8080")]
	/// the image port to bind to (default: 8080)
	pub image_port: u16,
}

impl CliArgs {
	async fn start_listening(host: &str, port: u16) -> TcpListener {
		TcpListener::bind(SocketAddrV4::new(
			host.parse().unwrap(),
			port,
		)).await.unwrap()
	}

	pub async fn start_listening_for_video(&self) -> TcpListener {
		Self::start_listening(&self.video_host, self.video_port).await
	}

	pub async fn start_listening_for_image(&self) -> TcpListener {
		Self::start_listening(&self.image_host, self.image_port).await
	}
}
