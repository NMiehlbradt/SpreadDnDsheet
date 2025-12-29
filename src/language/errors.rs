
#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn with_message<'a>(message: impl Into<String>) -> Self {
        Error {
            message: message.into(),
        }
    }

    pub fn propogated_error(cell_name: &str) -> Self {
        Error::with_message(format!("Error in read cell {}", cell_name))
    }
}
