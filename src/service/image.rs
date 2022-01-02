use std::convert::Infallible;
use std::net::SocketAddr;
use std::process::Stdio;

use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::header::HeaderName;
use hyper::http::HeaderValue;
use hyper::server::Builder;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::service::{make_service_fn, service_fn};
use tokio::process::Command;

use crate::utils::{SocksContainer, Writable};

#[derive(Clone)]
struct HyperCtx {
	socks: SocksContainer,
}

pub async fn listen_for_new_image_requests(server: Builder<AddrIncoming>, socks: SocksContainer) {
	let ctx = HyperCtx { socks };

	let make_service = make_service_fn(move |conn: &AddrStream| {
		let ctx = ctx.clone();
		let addr = conn.remote_addr();

		async move {
			Ok::<_, Infallible>(service_fn(move |req| {
				handle(ctx.clone(), addr, req)
			}))
		}
	});

	server
		.serve(make_service)
		.await
		.unwrap();
}

async fn handle(ctx: HyperCtx, addr: SocketAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
	let addr = addr.to_string();

	if req.method() != Method::GET {
		return bad_request();
	}

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
		ctx.socks.lock().await.insert(addr.clone(), Writable::ChildStdin(stdin));
	} else {
		error!("Failed to obtain stdin of ffmpeg for {}", addr);
		return internal_server_error();
	}

	if let Ok(output) = child.wait_with_output().await {
		if output.status.exit_ok().is_err() {
			error!("Caught an ffmpeg error for {}", addr);
			return internal_server_error();
		}

		Ok(Response::builder()
			.status(StatusCode::OK)
			.header(HeaderName::from_static("content-type"), HeaderValue::from_static("image/jpeg"))
			.body(Body::from(output.stdout))
			.unwrap())
	} else {
		error!("Failed to read ffmpeg capture for {}", addr);
		internal_server_error()
	}
}

fn bad_request() -> Result<Response<Body>, Infallible> {
	Ok(Response::builder()
		.status(StatusCode::BAD_REQUEST)
		.body(Body::empty())
		.unwrap())
}

fn internal_server_error() -> Result<Response<Body>, Infallible> {
	Ok(Response::builder()
		.status(StatusCode::INTERNAL_SERVER_ERROR)
		.body(Body::empty())
		.unwrap())
}
