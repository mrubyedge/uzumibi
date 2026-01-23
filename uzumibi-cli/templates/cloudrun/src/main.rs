use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub mod uzumibi;

async fn uzumibi_request(
    request: Request<IncomingBody>,
) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut uzumibi_request = uzumibi::build_uzumibi_request(&request);
    // HINT: The body must be collected independently because
    //       mruby/edge and uzumibi_gem structures are not `Send`.
    let body_bytes: Vec<u8> = request.into_body().collect().await?.to_bytes().to_vec();
    uzumibi_request.body = body_bytes;
    let response = uzumibi::uzumibi_handle_request(uzumibi_request)?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(uzumibi_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
