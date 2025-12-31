use fastly::{Error, Request, Response};
use $$PROJECT_NAME_UNDERSCORE$$::*;

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let sm = uzumibi_initialize_request(65536);
    sm.borrow_mut().write(0, &pack_request_data(&req));

    let res = uzumibi_start_request();

    Ok(res)
}
