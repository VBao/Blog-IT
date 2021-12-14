#[derive(PartialEq)]
pub enum ErrorMessage {
    NotOwned,
    NotFound,
    Unauthorized,
    ServerError,
    Duplicate,
    BadRequest,
}