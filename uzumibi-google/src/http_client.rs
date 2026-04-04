use reqwest::blocking::Client;
use std::sync::OnceLock;

// Keep one blocking client for process lifetime so its internal runtime
// is never dropped from an async worker context.
pub(crate) fn blocking_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}
