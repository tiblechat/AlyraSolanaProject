
#[error_code]
pub enum ErrorCode {
    #[msg("Account has already staked.")]
    AccountAlreadyStakedError,
    #[msg("Account has not deposited.")]
    InvalidAccountData,
}

use anchor_lang::prelude::error_code;

