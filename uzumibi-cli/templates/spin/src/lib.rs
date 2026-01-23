extern crate anyhow;

use anyhow::anyhow;
use spin_sdk::http::{IntoResponse, Request};
use spin_sdk::http_component;

pub mod uzumibi;

/// A simple Spin HTTP component.
#[http_component]
fn handle_uzumibi_on_spin_spike(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    uzumibi::uzumibi_initialize_request(req)
        .map_err(|e| anyhow!("Failed to initialize request: {}", e))?;
    uzumibi::uzumibi_start_request().map_err(|e| anyhow!("Failed to start request: {}", e))
}
