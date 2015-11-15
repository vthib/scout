#[derive(Debug)]
pub enum Error {
    TomlError(String),
    StructuralError(String),
}
