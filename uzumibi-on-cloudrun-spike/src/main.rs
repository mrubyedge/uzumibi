use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub mod uzumibi;

const SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

async fn uzumibi_request(
    request: Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    #[cfg(feature = "queue")]
    {
        let body_bytes: Vec<u8> = request.into_body().collect().await?.to_bytes().to_vec();
        eprintln!(
            "[uzumibi] queue message received, body size={}",
            body_bytes.len()
        );
        eprintln!(
            "[uzumibi] queue body: {}",
            String::from_utf8_lossy(&body_bytes)
        );
        let result = tokio::task::spawn_blocking(move || {
            uzumibi::uzumibi_dispatch_queue_message(&body_bytes).map_err(|e| e.to_string())
        })
        .await;
        match result {
            Ok(Ok(())) => {
                eprintln!("[uzumibi] queue message dispatched successfully");
                let response = Response::builder()
                    .status(200)
                    .body(Full::new(Bytes::from_static(b"ok")))?;
                return Ok(response);
            }
            Ok(Err(e)) => {
                eprintln!("[uzumibi] queue dispatch error: {}", e);
                let message = format!("Internal Server Error: {}", e);
                let response = Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from(message.into_bytes())))?;
                return Ok(response);
            }
            Err(e) => {
                eprintln!("[uzumibi] spawn_blocking failed: {}", e);
                let message = format!("Internal Server Error: spawn_blocking failed: {}", e);
                let response = Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from(message.into_bytes())))?;
                return Ok(response);
            }
        }
    }

    #[cfg(not(feature = "queue"))]
    {
        let mut uzumibi_request = uzumibi::build_uzumibi_request(&request);
        eprintln!(
            "[uzumibi] {} {}",
            uzumibi_request.method, uzumibi_request.path
        );
        // HINT: The body must be collected independently because
        //       mruby/edge and uzumibi_gem structures are not `Send`.
        let body_bytes: Vec<u8> = request.into_body().collect().await?.to_bytes().to_vec();
        uzumibi_request.body = body_bytes;

        let result = tokio::task::spawn_blocking(move || {
            uzumibi::uzumibi_handle_request(uzumibi_request).map_err(|e| e.to_string())
        })
        .await;

        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => {
                eprintln!("[uzumibi] request error: {}", e);
                let message = format!("Internal Server Error: {}", e);
                let response = Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from(message.into_bytes())))?;
                Ok(response)
            }
            Err(e) => {
                eprintln!("[uzumibi] spawn_blocking failed: {}", e);
                let message = format!("Internal Server Error: spawn_blocking failed: {}", e);
                let response = Response::builder()
                    .status(500)
                    .body(Full::new(Bytes::from(message.into_bytes())))?;
                Ok(response)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

    let graceful = hyper_util::server::graceful::GracefulShutdown::new();

    let listener = TcpListener::bind(addr).await?;
    eprintln!("[uzumibi] listening on http://{}", addr);
    println!("listening on http://{}", addr);

    loop {
        tokio::select! {
            Ok((stream, _addr)) =  listener.accept() => {
                let io = TokioIo::new(stream);
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(io, service_fn(uzumibi_request))
                        .await
                    {
                        eprintln!("error serving connection: {:?}", err);
                    }
                });

            }
            _ = sigterm.recv() => {
                println!("\nreceived SIGTERM, starting graceful shutdown");
                drop(listener);
                break;
            },
            _ = sigint.recv() => {
                println!("\nreceived SIGINT, starting graceful shutdown");
                drop(listener);
                break;
            },
        }
    }

    tokio::select! {
        _ = graceful.shutdown() => {
            println!("graceful shutdown completed");
        },
        _ = tokio::time::sleep(SHUTDOWN_TIMEOUT) => {
            Err("timed out wait for all connections to close")?;
        }
    }
    Ok(())
}
