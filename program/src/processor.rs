use solana_program::{account_info::{AccountInfo, next_account_info}, decode_error::DecodeError, entrypoint::ProgramResult, msg, program::{invoke, invoke_signed}, program_error::ProgramError, program_error::{PrintProgramError}, program_pack::Pack, pubkey::Pubkey, sysvar::{clock::Clock, Sysvar}};

use spl_token::{instruction::transfer, state::Account};
use spl_token::instruction::TokenInstruction;

use spl_associated_token_account::get_associated_token_address;
use num_traits::FromPrimitive;

use crate::{
    error::VestingError,
    instruction::VestingInstruction, 
    state::{VestingParameters, STATE_SIZE}
};


pub struct Processor {}

impl Processor {

    // pub fn process_init(
    //     program_id: &Pubkey, 
    //     accounts: &[AccountInfo], 
    //     seeds: [u8; 32], 
    //     amount: u64, 
    //     release_height: u64,
    //     mint_address: Pubkey
    // ) -> ProgramResult {        
    //     let accounts_iter = &mut accounts.iter();

    //     let system_account = next_account_info(accounts_iter)?;
    //     let vesting_account = next_account_info(accounts_iter)?;

    //     msg!("Key : {:?}", system_account.key);
    //     msg!("Vesting key : {:?}", vesting_account.key);
    //     // return Err(ProgramError::InvalidArgument);

    //     // if !system_account.executable {
    //     //     msg!("System account is executable!");
    //     //     return Err(ProgramError::InvalidArgument)
    //     // }

        
    //     // if system_account.is_writable {
    //     //     msg!("System account is writable!");
    //     //     return Err(ProgramError::InvalidArgument)
    //     // }


    //     // if *system_account.key != system_program::id(){
    //     //     return Err(ProgramError::InvalidArgument)
    //     // }


    //     let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;

    //     if vesting_account_key != *vesting_account.key {
    //         return Err(ProgramError::InvalidArgument)
    //     }

        

    //     // We might be able to do this with one invocation of allocate_with_seed
    //     invoke_signed(
    //         &allocate(&vesting_account_key, STATE_SIZE as u64),
    //         &[
    //             system_account.clone(),
    //             vesting_account.clone(),
    //         ],
    //         &[&[&seeds]]
    //     )?;

    //     invoke_signed(
    //         &assign(&vesting_account_key, program_id),
    //         &[
    //             system_account.clone(),
    //             vesting_account.clone()
    //         ],
    //         &[&[&seeds]]
    //     )?;

    //     let mut instruction_accounts:Vec<AccountMeta> = accounts
    //         .iter()
    //         .map(|a| AccountMeta::new(a.key.clone(), a.is_signer))
    //         .collect();
    //     instruction_accounts[2] = AccountMeta::new(vesting_account.key.clone(), true);


    //     let data = VestingInstruction::CreatePrivate { seeds, release_height, amount, mint_address }.pack();

    //     let instruction = Instruction { program_id: program_id.clone(), accounts:instruction_accounts, data };

    //     invoke_signed(
    //         &instruction,
    //         accounts,
    //         &[&[&seeds]]
    //     )?;



    //     Ok(())

    // }

    pub fn process_create(
        program_id: &Pubkey,
        accounts: &[AccountInfo], 
        seeds: [u8; 32], 
        amount: u64, 
        release_height: u64,
        mint_address: Pubkey,
        destination_token_address: Pubkey
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        //TODO put vesting and vesting token together
        let spl_token_account = next_account_info(accounts_iter)?;
        // let mint_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let source_token_account_owner = next_account_info(accounts_iter)?;
        let source_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Provided vesting account is invalud");
            return Err(ProgramError::InvalidArgument)
        }

        if !source_token_account_owner.is_signer {
            msg!("Source token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument)
        }

        if *vesting_account.owner != *program_id {
            msg!("Program should own vesting account");
            return Err(ProgramError::InvalidArgument)
        }

        // Verifying that no SVC was already created with this seed
        let is_initialized = vesting_account.try_borrow_data()?[STATE_SIZE-1] == 1;

        if is_initialized {
            msg!("Cannot overwrite an existing vesting contract.");
            return Err(ProgramError::InvalidArgument)
        }

        let state = VestingParameters { 
            destination_address: destination_token_address, 
            release_height, 
            mint_address: mint_address,
            amount,
            is_initialized: true
        };

        // TODO: Rework this
        let packed_state = state.pack();

        for i in 0..STATE_SIZE {
            vesting_account.try_borrow_mut_data()?[i] = packed_state[i];
        }

        let vesting_token_account_data = Account::unpack(
            &vesting_token_account.data.borrow()
        )?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument)
        }

        // let vesting_token_account_address = get_associated_token_address(vesting_account.key, &mint_address);
        // let source_token_account_address = get_associated_token_address(source_token_account_owner.key, &mint_address);

        // if *vesting_token_account.key != vesting_token_account_address {
        //     msg!("Invalid vesting token account provided");
        //     return Err(ProgramError::InvalidArgument)
        // }

        // if *source_token_account.key != source_token_account_address {
        //     msg!("Invalid source token account provided");
        //     return Err(ProgramError::InvalidArgument)
        // }

        let transfer_tokens_to_vesting_account = transfer(
            spl_token_account.key,
            source_token_account.key,
            vesting_token_account.key,
            source_token_account_owner.key,
            &[],
            amount
        )?;

        invoke(
            &transfer_tokens_to_vesting_account,
            &[
                source_token_account.clone(),
                vesting_token_account.clone(),  
                spl_token_account.clone(),
                // mint_account.clone(),
                source_token_account_owner.clone()
            ] // seed?
        )?;
        Ok(())
    }

    pub fn process_unlock(program_id: &Pubkey, _accounts: &[AccountInfo], seeds: [u8; 32]) -> ProgramResult {
        let accounts_iter = &mut _accounts.iter();
        
        let spl_token_account = next_account_info(accounts_iter)?;
        let clock_sysvar_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;

        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument)
        }
        
        let packed_state = vesting_account.try_borrow_data()?;
        let state = VestingParameters::unpack(packed_state.as_ref())?;
        
        if state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument)
        }

        let vesting_token_account_data = Account::unpack(
            &vesting_token_account.data.borrow()
        )?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument)
        }
        
        // let vesting_token_account_key = get_associated_token_address(&vesting_account_key, &state.mint_address);
        
        // if vesting_token_account_key != *vesting_token_account.key{
        //     msg!("Vesting token account does not match the provided vesting contract");
        //     return Err(ProgramError::InvalidArgument)
        // }

        // Check that sufficient slots have passed to unlock
        let clock = Clock::from_account_info(&clock_sysvar_account)?;
        if clock.slot < state.release_height {
            msg!("Vesting contract has not yet reached release time");
            return Err(ProgramError::InvalidArgument)
        }
        
        let transfer_tokens_from_vesting_account = transfer(
            &spl_token_account.key,
            &vesting_token_account.key,
            destination_token_account.key,
            &vesting_account_key,
            &[],
            state.amount                            //Could be done in cli
        )?;
        
        invoke_signed(
            &transfer_tokens_from_vesting_account,
            &[
                spl_token_account.clone(),
                vesting_token_account.clone(),
                destination_token_account.clone(),
                vesting_account.clone(),
                ],
                &[&[&seeds]]
            )?;
            
        Ok(())
    }

    pub fn process_change_destination(program_id: &Pubkey, accounts: &[AccountInfo], seeds: [u8; 32]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        
        let vesting_account = next_account_info(accounts_iter)?;
        let destination_token_info = next_account_info(accounts_iter)?;
        let destination_token_account_owner = next_account_info(accounts_iter)?;
        let new_destination_token_account = next_account_info(accounts_iter)?;

        
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        let state = VestingParameters::unpack(vesting_account.try_borrow_data()?.as_ref())?;

        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument)
        }
        
        if state.destination_address != *destination_token_info.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument)
        }
        
        if !destination_token_account_owner.is_signer {
            msg!("Destination token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument)
        }


        let destination_token_account = Account::unpack(
            &destination_token_info.data.borrow()
        )?;

        if destination_token_account.owner != *destination_token_account_owner.key {
            msg!("The current destination token account isn't owned by the provided owner");
            return Err(ProgramError::InvalidArgument)

        }
        
        let mut new_state = state;
        new_state.destination_address = *new_destination_token_account.key;
        let new_packed_state = new_state.pack();
        
        for i in 0..STATE_SIZE {
            vesting_account.try_borrow_mut_data()?[i] = new_packed_state[i];
        }

        Ok(())
    }


    pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Instruction unpacked");
        match instruction {
            VestingInstruction::Create { seeds, amount, release_height, mint_address, destination_token_address} => {
                msg!("Instruction: Create");
                Self::process_create(program_id, accounts, seeds, amount, release_height, mint_address, destination_token_address)
            }
            VestingInstruction::Unlock {seeds} => {
                msg!("Instruction: Unlock");
                Self::process_unlock(program_id, accounts, seeds)
            }
            VestingInstruction::ChangeDestination {seeds} => {
                msg!("Instruction: Change Destination");
                Self::process_change_destination(program_id, accounts, seeds)
            }
        }
    }
}

impl PrintProgramError for VestingError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            VestingError::InvalidInstruction => msg!("Error: Invalid instruction!")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create(){

        let mut seeds = [42u8;32];

        let source_account = Pubkey::new_unique();
        let mut source_lamports = 42u64;
        let mut destination_lamports = 10u64;
        let mut program_lamports = 0;
        let mut transaction_lamports = 0;
        let destination_account = Pubkey::new_unique();
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let (transaction, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);

        seeds[31] = bump;

        // let transaction = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();

        let mut transaction_data = [0u8;STATE_SIZE];


        let _accounts = vec![
            AccountInfo::new(
                &program_id,
                true,
                true,
                &mut program_lamports,
                &mut [],
                &owner,
                true,
                7000
            ),
            AccountInfo::new(
                &transaction,
                true,
                true,
                &mut transaction_lamports,
                &mut transaction_data,
                &owner,
                true,
                7000
            ),
            AccountInfo::new(
                &source_account,
                true,
                true,
                &mut source_lamports,
                &mut [],
                &owner,
                false,
                7000
            ),
            AccountInfo::new(
                &destination_account,
                true,
                true,
                &mut destination_lamports,
                &mut [],
                &owner,
                false,
                7000
            )
        ];
        // Processor::process_instruction(
        //     &program_id,
        //     &accounts,
        //     &VestingInstruction::Create {seeds, amount: 5, release_height: 0, mint_address: Pubkey::new_unique()}.pack()
        // ).unwrap();
        // assert_eq!(source_lamports, 37);
        // assert_eq!(transaction_lamports, 5);
    }
}