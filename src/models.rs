use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    // Make id optional because the form for creating a task does not send an id
    pub id: Option<i64>,
    pub title: String,
    pub completed: Option<bool>,
}
