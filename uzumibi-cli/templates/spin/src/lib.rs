use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

pub mod uzumibi;
use uzumibi::*;

/// A simple Spin HTTP component.
#[http_component]
fn handle_$$PROJECT_NAME_UNDERSCORE$$(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    let response = uzumibi_initialize_request(65536);
    let packed_request = pack_request_data(&req);
    response.borrow_mut().write(0, &packed_request);

    let ret: Response = uzumibi_start_request();
    Ok(ret)
}
