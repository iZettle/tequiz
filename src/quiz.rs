use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Quiz {
    pub question: String,
    pub answer: String,
    pub wrong_answers: Vec<String>,
}
