use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AiIntent {
    pub text: String,
    pub intent: String,
}
