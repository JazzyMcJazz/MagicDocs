use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum Permission {
    Read,
    Write,
    Delete,
}
