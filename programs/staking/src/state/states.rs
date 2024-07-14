use anchor_lang::prelude::*;






#[account]
#[derive(Default)] // will be init to zeros 
pub struct Receipt {
    pub is_valid: u8,
    pub created_ts: u64,
    pub amount_deposited: u64,
}



