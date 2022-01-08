use std::net::{SocketAddr, SocketAddrV4};
use std::str::FromStr;

use hyper::Server;
use hyper::server::Builder;
use hyper::server::conn::AddrIncoming;
use tokio::net::TcpListener;
use crate::model::camera::settings::{CameraProvider, Resolution, Rotation};

#[derive(FromArgs)]
/// serve multiple H.264 streams over TCP
pub struct CliArgs {
	#[argh(option, short = 'o', default = "SocketAddrV4::from_str(\"0.0.0.0:1264\").unwrap()")]
	/// the host (ipv4) to bind to for the video stream (default: 0.0.0.0:1264)
	pub video_host: SocketAddrV4,
	#[argh(option, short = 'h', default = "SocketAddrV4::from_str(\"0.0.0.0:8080\").unwrap()")]
	/// the host ip (ipv4) to bind to for the image http server (default: 0.0.0.0:8080)
	pub image_host: SocketAddrV4,
	#[argh(option, short = 'c', default = "Default::default()")]
	/// the camera stack to use (available options: "legacy" (alias "raspi"), "libcamera") (default: "legacy")
	pub camera_provider: CameraProvider,
	#[argh(option, short = 'v', default = "Default::default()")]
	/// the video resolution specified in units of pixel in the form of WxH (default: 1280x720)
	pub video_resolution: Resolution,
	#[argh(option, short = 'i', default = "Default::default()")]
	/// the image resolution specified in units of pixel in the form of WxH (default: 1280x720)
	pub image_resolution: Resolution,
	#[argh(option, short = 'r', default = "Default::default()")]
	/// the number of degrees the camera is offset by (available options: 0, 180 (90, 270 only available for legacy camera stack)) (default: 0)
	pub rotation: Rotation,
}

impl CliArgs {
	pub async fn start_listening_for_video(&self) -> TcpListener {
		TcpListener::bind(&self.video_host).await.unwrap()
	}

	pub async fn start_listening_for_image(&self) -> Builder<AddrIncoming> {
		Server::bind(&SocketAddr::V4(self.image_host))
	}
}
