use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum EchoError {
  #[error("Buffer consists of non-zero data")]
  BufferNonZero,
}

impl From<EchoError> for ProgramError {
  fn from(e: EchoError) -> Self {
    ProgramError::Custom(e as u32)
  }
}