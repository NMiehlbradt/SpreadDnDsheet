use crate::reactive::sheet::CellId;

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

    pub fn not_found(cell_id: &CellId) -> Self {
        Error::with_message(format!("Cell {} not found", cell_id.0))
    }

    pub fn propogated_error(cell_id: &CellId) -> Self {
        Error::with_message(format!("Error in read cell {}", cell_id.0))
    }
}
