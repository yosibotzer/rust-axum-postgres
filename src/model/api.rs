use serde::Deserialize;



#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub name: String,
    pub author: String
}

#[derive(Debug, Deserialize)]
pub struct QueryBookRequest {
    pub name: Option<String>,
    pub author: Option<String>
}