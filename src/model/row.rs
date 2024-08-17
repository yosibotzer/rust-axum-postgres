use serde::Serialize;
use sqlx::FromRow;


#[derive(Debug, Serialize, FromRow)]
pub struct BookRow {
    pub id: i32,
    pub name: String,
    pub author: String,
}
