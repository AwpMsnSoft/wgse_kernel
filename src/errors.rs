use thiserror::Error;
type Message = &'static str;

#[derive(Clone, Debug, Error)]
pub enum WgseUtilsError {
    #[error("inconsistent type, expect `{0}`, found `{1}`")]
    InconsistentType(Message, Message),
}
