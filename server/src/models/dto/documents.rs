use migration::sea_orm::{self, FromQueryResult};
use serde::Serialize;

#[derive(Debug, Serialize, FromQueryResult)]
pub struct DocumentWithIdAndName {
    pub id: i32,
    pub name: String,
}
