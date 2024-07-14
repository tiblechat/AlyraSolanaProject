mod constants;
mod error;

use anchor_lang::{
    prelude::*,
    solana_program::{clock::Clock, hash::hash, program::invoke, system_instruction::transfer},
};

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self,Mint, MintTo,Burn, Token, TokenAccount,Transfer as SplTransfer},   
};

use crate::{constants::*, error::LotteryError};

declare_id!("6ANiwC3FdEnXz9UgTX9haDmVuSxE4GxfRQ6DEke29exR");

#[program]
mod lottery {
    use super::*;
    pub fn init_master(_ctx: Context<InitMaster>) -> Result<()> {
        Ok(())
    }

    pub fn create_lottery(ctx: Context<CreateLottery>, ticket_price: u64) -> Result<()> {
        let master = &mut ctx.accounts.master;
        let lottery = &mut ctx.accounts.lottery;

        // Increment the last ticket id
        master.last_id += 1;

        // Set lottery values
        lottery.id = master.last_id;
        lottery.authority = ctx.accounts.authority.key();
        lottery.ticket_price = ticket_price;

        msg!("Created lottery: {}", lottery.id);
        msg!("Authority: {}", lottery.authority);
        msg!("Ticket price: {}", lottery.ticket_price);

        Ok(())
    }

    pub fn buy_ticket_with_token(ctx: Context<BuyTicketWithToken>, lottery_id: u32) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let ticket = &mut ctx.accounts.ticket;
        let buyer = &ctx.accounts.buyer;

        if lottery.winner_id.is_some() {
            return err!(LotteryError::WinnerAlreadyExists);
        }

        // Transfer TOKEN to Lottery PDA
        // 
        // invoke(
        //     &transfer(&buyer.key(), &lottery.key(), lottery.ticket_price),
        //     &[
        //         buyer.to_account_info(),
        //         lottery.to_account_info(),
        //         ctx.accounts.system_program.to_account_info(),
        //     ],
        // )?;
          // transfer X token from sender -> PDA vault 
          let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            SplTransfer {
                from: ctx.accounts.buyer_token_account.to_account_info().clone(), 
                to: ctx.accounts.lottery_token_account.to_account_info().clone(),
                authority: ctx.accounts.buyer.to_account_info().clone(), 
            }
        );
        token::transfer(transfer_ctx,  lottery.ticket_price)?;


        lottery.last_ticket_id += 1;

        ticket.id = lottery.last_ticket_id;
        ticket.lottery_id = lottery_id;
        ticket.authority = buyer.key();

        msg!("Ticket id: {}", ticket.id);
        msg!("Ticket authority: {}", ticket.authority);

        Ok(())
    }

    // TODO
    pub fn createStaker(ctx: Context<DepositStaking>, lottery_id: u32) -> Result<()> {
        //https://book.anchor-lang.com/anchor_in_depth/CPIs.html
        let cpi_program = ctx.accounts.staking.to_account_info();
        // create vault pda
        let cpi_accounts = new_staker {
            puppet: ctx.accounts.puppet.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        puppet::cpi::set_data(cpi_ctx, data)

    }

    pub fn depositToStaking(ctx: Context<DepositStaking>, lottery_id: u32) -> Result<()> {
        //https://book.anchor-lang.com/anchor_in_depth/CPIs.html
        let cpi_program = ctx.accounts.staking.to_account_info();
        // create vault pda
        let cpi_accounts = new_staker {
            puppet: ctx.accounts.puppet.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        puppet::cpi::set_data(cpi_ctx, data)

    }

    // pub fn removeFromStaking(ctx: Context<DepositStaking>, lottery_id: u32) -> Result<()> {
    // }

    // pub fn claimPrizeInToken(ctx: Context<DepositStaking>, lottery_id: u32) -> Result<()> {
    // }

    // pub fn refundTicket(ctx: Context<DepositStaking>, lottery_id: u32) -> Result<()> {
    // }

    // fin TODO
    pub fn buy_ticket(ctx: Context<BuyTicket>, lottery_id: u32) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let ticket = &mut ctx.accounts.ticket;
        let buyer = &ctx.accounts.buyer;

        if lottery.winner_id.is_some() {
            return err!(LotteryError::WinnerAlreadyExists);
        }

        // Transfer SOL to Lottery PDA
        invoke(
            &transfer(&buyer.key(), &lottery.key(), lottery.ticket_price),
            &[
                buyer.to_account_info(),
                lottery.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        lottery.last_ticket_id += 1;

        ticket.id = lottery.last_ticket_id;
        ticket.lottery_id = lottery_id;
        ticket.authority = buyer.key();

        msg!("Ticket id: {}", ticket.id);
        msg!("Ticket authority: {}", ticket.authority);

        Ok(())
    }

    pub fn pick_winner(ctx: Context<PickWinner>, _lottery_id: u32) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;

        if lottery.winner_id.is_some() {
            return err!(LotteryError::WinnerAlreadyExists);
        }
        if lottery.last_ticket_id == 0 {
            return err!(LotteryError::NoTickets);

        }

        // Pick a pseudo-random winner
        let clock = Clock::get()?;
        let pseudo_random_number = ((u64::from_le_bytes(
            <[u8; 8]>::try_from(&hash(&clock.unix_timestamp.to_be_bytes()).to_bytes()[..8])
                .unwrap(),
        ) * clock.slot)
            % u32::MAX as u64) as u32;

        let winner_id = (pseudo_random_number % lottery.last_ticket_id) + 1;
        lottery.winner_id = Some(winner_id);

        msg!("Winner id: {}", winner_id);

        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>, _lottery_id: u32, _ticket_id: u32) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let ticket = &ctx.accounts.ticket;
        let winner = &ctx.accounts.authority;

        if lottery.claimed {
            return err!(LotteryError::AlreadyClaimed);
        }

        // Validate winner_id
        match lottery.winner_id {
            Some(winner_id) => {
                if winner_id != ticket.id {
                    return err!(LotteryError::InvalidWinner);
                }
            }
            None => return err!(LotteryError::WinnerNotChosen),
        };

        // Transfer the prize from Lottery PDA to the winner
        let prize = lottery
            .ticket_price
            .checked_mul(lottery.last_ticket_id.into())
            .unwrap();

        **lottery.to_account_info().try_borrow_mut_lamports()? -= prize;
        **winner.to_account_info().try_borrow_mut_lamports()? += prize;

        lottery.claimed = true;

        msg!(
            "{} claimed {} lamports from lottery id {} with ticket id {}",
            winner.key(),
            prize,
            lottery.id,
            ticket.id
        );

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitMaster<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 4,
        seeds = [MASTER_SEED.as_bytes()],
        bump,
    )]
    pub master: Account<'info, Master>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Master {
    pub last_id: u32,
}

#[derive(Accounts)]
pub struct CreateLottery<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 4 + 32 + 8 + 4 + 1 + 4 + 1,
        seeds = [LOTTERY_SEED.as_bytes(), &(master.last_id + 1).to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(
        mut,
        seeds = [MASTER_SEED.as_bytes()],
        bump,
    )]
    pub master: Account<'info, Master>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Lottery {
    pub id: u32,
    pub authority: Pubkey,
    pub ticket_price: u64,
    pub last_ticket_id: u32,
    pub winner_id: Option<u32>,
    pub claimed: bool,
}

#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct BuyTicketWithToken<'info> {
    #[account(
        mut,
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(
        init,
        payer = buyer,
        space = 8 + 4 + 4 + 32,
        seeds = [
            TICKET_SEED.as_bytes(),
            lottery.key().as_ref(),
            &(lottery.last_ticket_id + 1).to_le_bytes()
        ],
        bump,
    )]
    pub ticket: Account<'info, Ticket>,

    pub lottery_token_account: Account<'info, TokenAccount>,//pda 
    pub buyer_token_account: Account<'info, TokenAccount>,//pda
    pub token: Account<'info, Mint>,// token

    #[account(mut)]
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct BuyTicket<'info> {
    #[account(
        mut,
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(
        init,
        payer = buyer,
        space = 8 + 4 + 4 + 32,
        seeds = [
            TICKET_SEED.as_bytes(),
            lottery.key().as_ref(),
            &(lottery.last_ticket_id + 1).to_le_bytes()
        ],
        bump,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Ticket {
    pub id: u32,
    pub lottery_id: u32,
    pub authority: Pubkey,
}

#[derive(Accounts)]
#[instruction(lottery_id: u32)]
pub struct PickWinner<'info> {
    #[account(
        mut,
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump,
        has_one = authority,
    )]
    pub lottery: Account<'info, Lottery>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(lottery_id: u32, ticket_id: u32)]
pub struct ClaimPrize<'info> {
    #[account(
        mut,
        seeds = [LOTTERY_SEED.as_bytes(), &lottery_id.to_le_bytes()],
        bump,
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(
        seeds = [
            TICKET_SEED.as_bytes(),
            lottery.key().as_ref(),
            &ticket_id.to_le_bytes()
        ],
        bump,
        has_one = authority,
    )]
    pub ticket: Account<'info, Ticket>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}