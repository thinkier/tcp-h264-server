use tokio::net::TcpListener;

use crate::utils::{ Writable};
use crate::VideoWrapper;

pub async fn listen_for_new_video_sockets(listener: TcpListener, vw: VideoWrapper) {
	while let Ok((client, addr)) = listener.accept().await {
		vw.register(addr.to_string(), Writable::TcpStream(client)).await;
	}
}
