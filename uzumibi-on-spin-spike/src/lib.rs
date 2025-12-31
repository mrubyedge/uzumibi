use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

pub mod uzumibi;

/// A simple Spin HTTP component.
#[http_component]
fn handle_uzumibi_on_spin_spike(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    let response = uzumibi::uzumibi_initialize_request(65536);
    let packed_request = uzumibi::pack_request_data(&req);
    response.borrow_mut().write(0, &packed_request);

    let ret: Response = uzumibi::uzumibi_start_request();
    Ok(ret)
}
