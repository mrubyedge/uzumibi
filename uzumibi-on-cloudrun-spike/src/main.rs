use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use time::OffsetDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use tokio::net::TcpListener;

#[cfg(feature = "queue")]
use uzumibi_google::QueueDispatchResult;

pub mod uzumibi;

const SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const NGINX_TIME_FORMAT: &[BorrowedFormatItem<'static>] = format_description!(
    "[day]/[month repr:short]/[year]:[hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
);

fn now_for_nginx_log() -> String {
    if let Ok(now) = OffsetDateTime::now_local() {
        return now
            .format(NGINX_TIME_FORMAT)
            .unwrap_or_else(|_| "-".to_string());
    }
    OffsetDateTime::now_utc()
        .format(NGINX_TIME_FORMAT)
        .unwrap_or_else(|_| "-".to_string())
}

async fn uzumibi_request(
    request: Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    #[cfg(feature = "queue")]
    {
        let body_bytes: Vec<u8> = request.into_body().collect().await?.to_bytes().to_vec();
        let result = tokio::task::spawn_blocking(move || {
            uzumibi::uzumibi_dispatch_queue_message(&body_bytes)
        })
        .await;

        let (status_code, body_bytes) = match result {
            Ok(QueueDispatchResult::Ack) => (200, Bytes::from_static(b"ok")),
            Ok(QueueDispatchResult::Redeliver) => (500, Bytes::from_static(b"redeliver")),
            Ok(QueueDispatchResult::InternalError(e)) => {
                let message = format!("Internal Server Error: {}", e);
                (500, Bytes::from(message))
            }
            Err(e) => {
                let message = format!("Internal Server Error: spawn_blocking failed: {}", e);
                (500, Bytes::from(message))
            }
        };

        let body_size = body_bytes.len();
        let response = Response::builder()
            .status(status_code)
            .body(Full::new(body_bytes))?;

        let now = now_for_nginx_log();
        eprintln!(
            "- - - [{}] \"{} {} {:?}\" {} {}",
            now, method, uri, version, status_code, body_size
        );

        Ok(response)
    }

    #[cfg(not(feature = "queue"))]
    {
        use hyper::body::Body;

        let mut uzumibi_request = uzumibi::build_uzumibi_request(&request);
        // HINT: The body must be collected independently because
        //       mruby/edge and uzumibi_gem structures are not `Send`.
        let body_bytes: Vec<u8> = request.into_body().collect().await?.to_bytes().to_vec();
        uzumibi_request.body = body_bytes;

        let result = tokio::task::spawn_blocking(move || {
            uzumibi::uzumibi_handle_request(uzumibi_request).map_err(|e| e.to_string())
        })
        .await;

        let (status_code, body_bytes) = match result {
            Ok(Ok(response)) => {
                // Extract status from the response, then reconstruct it
                let status = response.status();
                let (parts, body_full) = response.into_parts();

                // For the response body, we'll work with it directly
                // For logging purposes, we log with 0 body size since we're not reading it
                let status_code = status.as_u16();

                // Return response parts to reconstruct response later
                (status_code, (parts, body_full))
            }
            Ok(Err(_e)) => {
                let message = Bytes::from("Internal Server Error");
                let status_code = 500;
                let response = Response::builder()
                    .status(status_code)
                    .body(Full::new(message.clone()))?;
                let (parts, body_full) = response.into_parts();
                (status_code, (parts, body_full))
            }
            Err(_e) => {
                let message = Bytes::from("Internal Server Error: spawn_blocking failed");
                let status_code = 500;
                let response = Response::builder()
                    .status(status_code)
                    .body(Full::new(message.clone()))?;
                let (parts, body_full) = response.into_parts();
                (status_code, (parts, body_full))
            }
        };

        let (parts, body_full) = body_bytes;

        let hint = body_full.size_hint();
        let hinted_size = hint
            .exact()
            .map(|size| size.to_string())
            .unwrap_or("-".to_string());
        let now = now_for_nginx_log();
        eprintln!(
            "- - - [{}] \"{} {} {:?}\" {} {}",
            now, method, uri, version, status_code, hinted_size
        );
        let response = Response::from_parts(parts, body_full);
        Ok(response)
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
