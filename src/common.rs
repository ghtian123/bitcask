use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum response {
    Ok(Option<String>),
    Err(String),
}
