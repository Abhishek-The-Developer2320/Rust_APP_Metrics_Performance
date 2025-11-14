use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    // Make id optional because the form for creating a task does not send an id
    pub id: Option<i64>,
    pub title: String,
    // Make completed optional to handle unchecked checkboxes (which are not submitted)
    pub completed: Option<bool>,
}
