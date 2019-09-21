use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RetryPayload {
    pub reference: String,
    pub retries: u32,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub request_url: String,
    pub payload: String,
}
