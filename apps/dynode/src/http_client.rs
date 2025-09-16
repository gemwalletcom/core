use reqwest::Client;

pub type HttpClient = Client;

pub fn new() -> HttpClient {
    Client::new()
}
