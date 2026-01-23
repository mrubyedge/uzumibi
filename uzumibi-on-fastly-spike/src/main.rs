// use fastly::http::StatusCode;
extern crate anyhow;

use anyhow::anyhow;
use fastly::{Error, Request, Response};
use uzumibi_on_fastly_spike as uzumibi;

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    uzumibi::uzumibi_initialize_request(req)
        .map_err(|e| anyhow!("Failed to initialize request: {}", e))?;
    uzumibi::uzumibi_start_request().map_err(|e| anyhow!("Failed to start request: {}", e))
}
