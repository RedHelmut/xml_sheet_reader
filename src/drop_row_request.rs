#[derive(Clone)]
pub enum DropRowRequest {
    Error(String),
    Ignore
}