use serde::Serialize;
use specta::Type;

use crate::extensions::AnyhowErrorToStringChain;

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Type)]
pub struct CommandError(String);
impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#}", self.0))
    }
}
impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        Self(err.to_string_chain())
    }
}
