use borsh::{BorshDeserialize};
use solana_program::{
    account_info::{AccountInfo}, 
    entrypoint::ProgramResult, 
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction;
// use crate::state::Echo;

pub struct Processor {}

impl Processor {
  pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {
    let instruction = instruction::EchoInstruction::try_from_slice(instruction_data)
      .map_err(|_| ProgramError::InvalidAccountData)?;

    let res = match instruction {
      instruction::EchoInstruction::Echo{ data } => {
        instruction::echo_ix(
          *program_id,
          accounts,
          data,
        )
      },
      instruction::EchoInstruction::InitializeAuthorizedEcho{ buffer_seed, buffer_size } => {
        instruction::init_auth_echo_ix(
          *program_id,
          accounts,
          buffer_seed,
          buffer_size,
        )
      },
      instruction::EchoInstruction::AuthorizedEcho{ data } => {
        instruction::auth_echo_ix(
          *program_id,
          accounts,
          data,
        )
      },
      instruction::EchoInstruction::InitializeVendingMachine{ price, buffer_size } => {
        instruction::init_vending_echo_ix(
          *program_id,
          accounts,
          price,
          buffer_size,
        )
        // Ok(())
      },
      instruction::EchoInstruction::VendingMachineEcho{ data } => {
        instruction::vending_echo_ix(
          *program_id,
          accounts,
          data,
        )
        // Ok(())
      }
    };

    res
  }
}