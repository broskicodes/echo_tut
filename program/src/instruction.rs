use borsh::{ BorshSerialize, BorshDeserialize };
use solana_program::{
  // instruction::Instruction, 
  program_error::ProgramError,
  entrypoint::ProgramResult,
  pubkey::Pubkey,
  account_info::{next_account_info, AccountInfo},
  system_instruction::create_account,
  sysvar::rent::Rent,
  program::{invoke, invoke_signed},
  system_program::ID as SYSTEM_PROGRAM_ID,
  msg,
};
use spl_token::{
  ID as TOKEN_PROGRAM_ID,
  instruction::burn,
};
use crate::error;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum EchoInstruction {
  Echo {
    data: Vec<u8>
  },
  InitializeAuthorizedEcho {
    buffer_seed: u64,
    buffer_size: usize,
  },
  AuthorizedEcho {
    data: Vec<u8>
  },
  InitializeVendingMachine {
    price: u64,
    buffer_size: usize,
  },
  VendingMachineEcho {
    data: Vec<u8>,
  },
}

pub fn assert_with_msg(statement: bool, err: ProgramError, msg: &str) -> ProgramResult {
  if !statement {
      msg!(msg);
      Err(err)
  } else {
      Ok(())
  }
}

pub fn echo_ix(
  _program_id: Pubkey,
  accounts: &[AccountInfo],
  data: Vec<u8>,
) -> ProgramResult {
  msg!("Instruction: Echo");
  let accounts_iter = &mut accounts.iter();
  let echo_info = next_account_info(accounts_iter)?;

  let prev_data = &mut *echo_info.data.borrow_mut();
  let sum: u8 = prev_data.iter().sum();

  assert_with_msg(
    sum == 0, 
    ProgramError::from(error::EchoError::BufferNonZero),
    "Buffer consists of non-zero data",
  )?;

  msg!("Updating buf");
  // let data_buf = &mut *echo_info.data.borrow_mut();
  match data.len() < prev_data.len()-4 {
    true => data.serialize(prev_data)?,
    false => data[0..prev_data.len()-4].serialize(prev_data)?,
  }

  Ok(())
}

pub fn init_auth_echo_ix(
  program_id: Pubkey,
  accounts: &[AccountInfo],
  buffer_seed: u64,
  buffer_size: usize,
) -> Result<(), ProgramError> {
  msg!("Instruction: Initialize Authorized Echo");

  let accounts_iter = &mut accounts.iter();
  let auth_buffer_info = next_account_info(accounts_iter)?;
  let authority = next_account_info(accounts_iter)?;
  let system_program = next_account_info(accounts_iter)?;

  assert_with_msg(
    authority.is_signer,
    ProgramError::MissingRequiredSignature,
    "Authority must sign",
  )?;

  let (auth_buffer_key, bump) = Pubkey::find_program_address(
    &[
      b"authority",
      authority.key.as_ref(),
      &buffer_seed.to_le_bytes(),
    ],
    &program_id,
  );

  assert_with_msg(
    *system_program.key == SYSTEM_PROGRAM_ID,
    ProgramError::InvalidArgument,
    "Invalid system program passed",
  )?;

  assert_with_msg(
    auth_buffer_key == *auth_buffer_info.key,
    ProgramError::InvalidArgument,
    "Provided PDA has incorrect seeds",
  )?;

  let create_ix = create_account(
    authority.key,
    &auth_buffer_key,
    Rent::default().minimum_balance(buffer_size),
    buffer_size as u64,
    &program_id,
  );

  invoke_signed(
    &create_ix,
    &[authority.clone(), auth_buffer_info.clone(), system_program.clone()],
    &[&[b"authority", authority.key.as_ref(), &buffer_seed.to_le_bytes(), &[bump]]],
  )?;

  let mut data = vec![bump];
  let buf_seed_arr = buffer_seed.to_le_bytes();

  for i in 0..7 {
    data.push(buf_seed_arr[i]);
  }

  data.serialize(&mut *auth_buffer_info.data.borrow_mut())?;

  Ok(())
}

pub fn auth_echo_ix(
  program_id: Pubkey,
  accounts: &[AccountInfo],
  data: Vec<u8>,
) -> ProgramResult {
  msg!("Instruction: Authorized Echo");
  let accounts_iter = &mut accounts.iter();
  let auth_buffer_info = next_account_info(accounts_iter)?;
  let authority = next_account_info(accounts_iter)?;

  assert_with_msg(
    authority.is_signer,
    ProgramError::MissingRequiredSignature,
    "Authority must sign",
  )?;

  let prev_data = &mut *auth_buffer_info.data.borrow_mut();
  let (auth_buffer_key, _) = Pubkey::find_program_address(
    &[
      b"authority",
      authority.key.as_ref(),
      &prev_data[5..13],
    ],
    &program_id,
  );

  assert_with_msg(
    auth_buffer_key == *auth_buffer_info.key,
    ProgramError::InvalidArgument,
    "Provided PDA has incorrect seeds",
  )?;

  let write_data = [prev_data[4..13].to_vec(), data].concat();

  match write_data.len() < prev_data.len()-4 {
    true => write_data.serialize(prev_data)?,
    false => write_data[0..prev_data.len()-4].serialize(prev_data)?,
  }

  Ok(())
}

pub fn init_vending_echo_ix(
  program_id: Pubkey,
  accounts: &[AccountInfo],
  price: u64,
  buffer_size: usize,
) -> Result<(), ProgramError> {
  msg!("Instruction: Initialize Vending Echo");

  let accounts_iter = &mut accounts.iter();
  let vend_buffer_info = next_account_info(accounts_iter)?;
  let vending_machine_mint = next_account_info(accounts_iter)?;
  let payer = next_account_info(accounts_iter)?;
  let system_program = next_account_info(accounts_iter)?;

  assert_with_msg(
    payer.is_signer,
    ProgramError::MissingRequiredSignature,
    "Payer must sign",
  )?;

  let (auth_buffer_key, bump) = Pubkey::find_program_address(
    &[
      b"vending machine",
      vending_machine_mint.key.as_ref(),
      &price.to_le_bytes(),
    ],
    &program_id,
  );

  assert_with_msg(
    *system_program.key == SYSTEM_PROGRAM_ID,
    ProgramError::InvalidArgument,
    "Invalid system program passed",
  )?;

  assert_with_msg(
    auth_buffer_key == *vend_buffer_info.key,
    ProgramError::InvalidArgument,
    "Provided PDA has incorrect seeds",
  )?;

  let create_ix = create_account(
    payer.key,
    &auth_buffer_key,
    Rent::default().minimum_balance(buffer_size),
    buffer_size as u64,
    &program_id,
  );

  invoke_signed(
    &create_ix,
    &[payer.clone(), vend_buffer_info.clone(), system_program.clone()],
    &[&[b"vending machine", vending_machine_mint.key.as_ref(), &price.to_le_bytes(), &[bump]]],
  )?;

  let mut data = vec![bump];
  let buf_seed_arr = price.to_le_bytes();

  for i in 0..7 {
    data.push(buf_seed_arr[i]);
  }

  data.serialize(&mut *vend_buffer_info.data.borrow_mut())?;

  Ok(())
}

pub fn vending_echo_ix(
  program_id: Pubkey,
  accounts: &[AccountInfo],
  data: Vec<u8>,
) -> ProgramResult {
  msg!("Instruction: Vending Echo");
  let accounts_iter = &mut accounts.iter();
  let vend_buffer_info = next_account_info(accounts_iter)?;
  let user = next_account_info(accounts_iter)?;
  let user_token_account = next_account_info(accounts_iter)?;
  let vending_machine_mint = next_account_info(accounts_iter)?;
  let token_program = next_account_info(accounts_iter)?;

  assert_with_msg(
    user.is_signer,
    ProgramError::MissingRequiredSignature,
    "Authority must sign",
  )?;

  assert_with_msg(
    *token_program.key == TOKEN_PROGRAM_ID,
    ProgramError::InvalidArgument,
    "Invalid system program passed",
  )?;

  let prev_data = &mut *vend_buffer_info.data.borrow_mut();
  let (auth_buffer_key, _) = Pubkey::find_program_address(
    &[
      b"vending machine",
      vending_machine_mint.key.as_ref(),
      &prev_data[5..13],
    ],
    &program_id,
  );

  assert_with_msg(
    auth_buffer_key == *vend_buffer_info.key,
    ProgramError::InvalidArgument,
    "Provided PDA has incorrect seeds",
  )?;

  let mut arr: [u8; 8] = [0; 8];
  for i in 5..13 {
    arr[i-5] = prev_data[i];
  }
  let price: u64 = u64::from_le_bytes(arr);

  let burn_ix = burn(
    token_program.key,
    user_token_account.key,
    vending_machine_mint.key,
    user.key,
    &[user.key],
    price,
  )?;

  invoke(
    &burn_ix,
    &[
      token_program.clone(), 
      user_token_account.clone(),
      vending_machine_mint.clone(),
      user.clone(),
    ],
  )?;

  let write_data = [prev_data[4..13].to_vec(), data].concat();

  match write_data.len() < prev_data.len()-4 {
    true => write_data.serialize(prev_data)?,
    false => write_data[0..prev_data.len()-4].serialize(prev_data)?,
  }

  Ok(())
}