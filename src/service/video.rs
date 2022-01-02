use std::time::Duration;

use tokio::net::TcpListener;
use tokio::time::interval;

use crate::utils::{SocksContainer, Writable};

pub async fn listen_for_new_video_sockets(listener: TcpListener, socks: SocksContainer) {
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
