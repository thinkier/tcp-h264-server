use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;

use crate::utils::{SocksContainer, Writable};

pub async fn listen_for_new_image_requests(listener: TcpListener, socks: SocksContainer) {
	// TODO Replace with hyper server
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
				let _ = client.flush().await;
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
	let _ = client.flush().await;
}
