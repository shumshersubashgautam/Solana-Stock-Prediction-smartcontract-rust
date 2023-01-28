//prediction dapp
//get the price of the stock pyth network
//player A to choose an asset that available in pyth and inputs the length of the bet
//players can see all existing bets(Fetch the bets)
//player B can match any bet and inputs his own prediction
//if the timing is over whoever is close to the prediction can claim the Whole amount

mod constants;
mod error;
mod state;
mod utils;

use anchor_lang::{prelude::*, system_program};
use pyth_sdk_solana::load_price_feed_from_account_info;

use crate::{constants::*, error::*, state::*, utils::*};

declare_id!("EWELwokyZpj3jME78cqAb35F6tKkdBthp3ohd3hGcFbu");

#[program]
mod prediction_dapp {
    use super::*;

    pub fn create_master(_ctx: Context<CreateMaster>) -> Result<()> {
        Ok(())
    }
    pub fn create_bet(
        ctx: Context<CreateBet>,
        amount: u64,
        price: f64,
        duration: u32,
        pyth_price_key: Pubkey,
    ) -> Result<()> {
        let master = &mut ctx.accounts.master;
        let bet = &mut ctx.accounts.bet;
        //Increase the last id on each bet creation on the master
        master.last_bet_id += 1;
        bet.id = master.last_bet_id;
        bet.pyth_price_key = pyth_price_key;
        bet.amount = amount;
        bet.expiry_ts = get_unix_timestamp() + duration as i64;
        bet.prediction_a = BetPrediction {
            player: ctx.accounts.player.key(),
            price,
        };

        //transfer the amount to the BET PDA

        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.player.to_account_info(),
                    to: bet.to_account_info(),
                },
            ),
            bet.amount,
        )?;

        Ok(())
    }

    pub fn enter_bet(ctx: Context<EnterBet>, price: f64) -> Result<()> {
        let bet = &mut ctx.accounts.bet;
        bet.prediction_b = Some(BetPrediction {
            player: ctx.accounts.player.key(),
            price,
        });
        bet.state = BetState::Started;
        //transfer the amount to the bet pda
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.player.to_account_info(),
                    to: bet.to_account_info(),
                },
            ),
            bet.amount,
        )?;

        Ok(())
    }

    pub fn claim_bet(ctx: Context<ClaimBet>) -> Result<()> {
        //check winner and send the prize to the winner or return them back if its a draw
        let bet = &mut ctx.accounts.bet;
        let prize = bet.amount.checked_mul(2).unwrap();
        **bet.to_account_info().try_borrow_mut_lamports()? -= prize;
        //get pyth price
        let pyth_account_info = &ctx.accounts.pyth;
        let feed = load_price_feed_from_account_info(pyth_account_info)
            .map_err(|_| error!(BetError::InvalidPythAccount))?;
        let price_data = feed.get_price_unchecked();
        require!(price_data.price <= f64::max as i64, BetError::PriceTooBig);
        let pyth_price = price_data.price as f64;
        msg!("Pyth price is : {}", pyth_price);

        //adjust pricess to compare them with pyths price
        //real price = pyth price *10(pyth exponent)
        let multiplier = 10f64.powi(-price_data.expo);
        let adjusted_player_a = bet.prediction_a.price * multiplier;
        let adjusted_player_b = bet.prediction_b.as_ref().unwrap().price * multiplier;
        msg!("Adjusted player A prediction: {}", adjusted_player_a);
        msg!("Adjusted player B prediction : {}", adjusted_player_b);
        let abs_player_a = (pyth_price - adjusted_player_a).abs();
        let abs_player_b = (pyth_price - adjusted_player_b).abs();
        if abs_player_a < abs_player_b {
            msg!("Winner is Player A, sending {} lamports", prize);
            bet.state = BetState::PlayerAWon;
            **ctx
                .accounts
                .player_a
                .to_account_info()
                .try_borrow_mut_lamports()? += prize;
        } else if abs_player_b < abs_player_a {
            msg!("Winner is Player B, sending {} lamports", prize);
            bet.state = BetState::PlayerBWon;
            **ctx
                .accounts
                .player_b
                .to_account_info()
                .try_borrow_mut_lamports()? += prize;
        } else {
            let draw_amount = bet.amount;
            msg!("Draw! Sending both player {} lamports", draw_amount);
            bet.state = BetState::Draw;

            //Return both players amount back
            **ctx
                .accounts
                .player_a
                .to_account_info()
                .try_borrow_mut_lamports()? += draw_amount;
            **ctx
                .accounts
                .player_b
                .to_account_info()
                .try_borrow_mut_lamports()? += draw_amount;
        }

        Ok(())
    }
    pub fn close_bet(_ctx: Context<CloseBet>) -> Result<()>{
        Ok(()) 
    }
}

#[derive(Accounts)]
pub struct CreateMaster<'info> {
    #[account(
         init,
         payer = payer,
         space = 8 + 8,
         seeds = [MASTER_SEED],
         bump
     )]
    pub master: Account<'info, Master>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]

pub struct CreateBet<'info> {
    #[account(
         init,
         payer = player,
         space = 8 + 8 + 32 + 8 + 8 + 32 + 8 + 1 + 32 + 8 + 1,
         seeds = [BET_SEED, &(master.last_bet_id +1).to_le_bytes()],
         bump
     )]
    pub bet: Account<'info, Bet>,

    #[account(mut, seeds = [MASTER_SEED], bump)]
    pub master: Account<'info, Master>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EnterBet<'info> {
    #[account(
        mut,
        seeds =[BET_SEED, &bet.id.to_le_bytes()],
        bump,
        constraint = validate_enter_bet(&*bet) @ BetError :: CannotEnter
    )]
    pub bet: Account<'info, Bet>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimBet<'info> {
    #[account(
        mut,
        seeds = [BET_SEED, &bet.id.to_le_bytes()],
        bump,
        constraint = validate_claim_bet(&*bet) @ BetError::CannotClaim,
    )]
    pub bet: Account<'info, Bet>,

    #[account(address = bet.pyth_price_key @ BetError::InvalidPythKey)]
    pub pyth: AccountInfo<'info>,
    #[account(mut,address = bet.prediction_a.player)]
    pub player_a: AccountInfo<'info>,

    #[account(mut,address = bet.prediction_b.as_ref().unwrap().player)]
    pub player_b: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseBet<'info> {
    #[account(
        mut,
        seeds = [BET_SEED, &bet.id.to_le_bytes()],
        bump,
        close = player,
        constraint = validate_close_bet(&*bet,player.key()) @BetError ::CannotClose,
        
    )]
    pub bet:Account<'info,Bet>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program:Program<'info,System>,
} 