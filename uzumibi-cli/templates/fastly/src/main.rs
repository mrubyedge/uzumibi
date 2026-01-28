extern crate anyhow;

use anyhow::anyhow;
use fastly::{Error, Request, Response};
use $$PROJECT_NAME_UNDERSCORE$$ as uzumibi;

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    uzumibi::uzumibi_initialize_request(req)
        .map_err(|e| anyhow!("Failed to initialize request: {}\n", e))?;
    uzumibi::uzumibi_start_request().map_err(|e| anyhow!("Failed to start request: {}\n", e))
}
