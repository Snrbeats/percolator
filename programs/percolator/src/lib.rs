use anchor_lang::prelude::*;

declare_id!("Percolator1111111111111111111111111111111");

#[program]
pub mod percolator {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.state.vault_balance = 0;
        ctx.accounts.state.insurance_fund = 0;
        ctx.accounts.state.total_capital = 0;
        ctx.accounts.state.total_pnl = 0;
        ctx.accounts.state.coverage_ratio_bps = 10000;
        ctx.accounts.state.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);
        
        ctx.accounts.state.total_capital += amount;
        ctx.accounts.state.vault_balance += amount;
        ctx.accounts.user.capital += amount;
        
        update_coverage_ratio(&mut ctx.accounts.state);
        
        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            amount,
        });
        
        Ok(())
    }

    pub fn update_pnl(ctx: Context<UpdatePnl>, pnl_change: i64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        
        if user.pnl > 0 && pnl_change < 0 {
            let reduction = (-pnl_change) as u64;
            ctx.accounts.state.total_pnl = ctx.accounts.state.total_pnl.saturating_sub(
                reduction.min(ctx.accounts.state.total_pnl)
            );
        }
        
        let old_pnl = user.pnl;
        user.pnl = user.pnl.saturating_add(pnl_change);
        
        if user.pnl > 0 && old_pnl <= 0 {
            ctx.accounts.state.total_pnl += user.pnl as u64;
        }
        
        update_coverage_ratio(&mut ctx.accounts.state);
        
        emit!(TradeEvent {
            user: user.key(),
            pnl_change,
        });
        
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);
        
        let state = &mut ctx.accounts.state;
        let user = &mut ctx.accounts.user;
        
        let withdrawable = calculate_withdrawable(
            user.capital,
            user.pnl,
            state.coverage_ratio_bps
        );
        
        require!(amount <= withdrawable, CustomError::InsufficientBalance);
        
        let capital_withdrawn = amount.min(user.capital);
        user.capital = user.capital.saturating_sub(capital_withdrawn);
        state.total_capital = state.total_capital.saturating_sub(capital_withdrawn);
        
        let remaining = amount.saturating_sub(capital_withdrawn);
        if remaining > 0 && user.pnl > 0 {
            let max_profit = ((user.pnl as u64) * state.coverage_ratio_bps as u64) / 10000;
            let profit_withdrawn = remaining.min(max_profit);
            user.pnl = user.pnl.saturating_sub(profit_withdrawn as i64);
            state.total_pnl = state.total_pnl.saturating_sub(profit_withdrawn);
        }
        
        state.vault_balance = state.vault_balance.saturating_sub(amount);
        
        update_coverage_ratio(state);
        
        emit!(WithdrawalEvent {
            user: user.key(),
            amount,
        });
        
        Ok(())
    }

    pub fn add_insurance(ctx: Context<AddInsurance>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);
        
        ctx.accounts.state.insurance_fund += amount;
        ctx.accounts.state.vault_balance += amount;
        
        update_coverage_ratio(&mut ctx.accounts.state);
        
        Ok(())
    }

    pub fn get_state(ctx: Context<GetState>) -> Result<StateResponse> {
        let state = &ctx.accounts.state;
        Ok(StateResponse {
            vault_balance: state.vault_balance,
            insurance_fund: state.insurance_fund,
            total_capital: state.total_capital,
            total_pnl: state.total_pnl,
            coverage_ratio_bps: state.coverage_ratio_bps,
        })
    }
}

fn update_coverage_ratio(state: &mut Account<State>) {
    state.coverage_ratio_bps = calculate_coverage_ratio(
        state.vault_balance,
        state.total_capital,
        state.insurance_fund,
        state.total_pnl,
    );
}

fn calculate_coverage_ratio(
    vault_balance: u64,
    total_capital: u64,
    insurance_fund: u64,
    total_pnl: u64,
) -> u16 {
    if total_pnl == 0 {
        return 10000;
    }
    
    let residual = vault_balance
        .saturating_sub(total_capital)
        .saturating_sub(insurance_fund);
    
    if residual >= total_pnl {
        return 10000;
    }
    
    ((residual * 10000) / total_pnl) as u16
}

fn calculate_withdrawable(capital: u64, pnl: i64, coverage_ratio_bps: u16) -> u64 {
    let profit = if pnl > 0 { 
        ((pnl as u64) * coverage_ratio_bps as u64) / 10000 
    } else { 
        0 
    };
    capital + profit
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 48)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(init, payer = user, space = 8 + 40, seeds = [b"user", user.key().as_ref()], bump)]
    pub user: Account<'info, UserAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePnl<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user: Account<'info, UserAccount>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut, seeds = [b"user", user.key().as_ref()], bump)]
    pub user: Account<'info, UserAccount>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddInsurance<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetState<'info> {
    pub state: Account<'info, State>,
}

#[account]
pub struct State {
    pub vault_balance: u64,
    pub insurance_fund: u64,
    pub total_capital: u64,
    pub total_pnl: u64,
    pub coverage_ratio_bps: u16,
    pub authority: Pubkey,
}

#[account]
#[default]
pub struct UserAccount {
    pub owner: Pubkey,
    pub capital: u64,
    pub pnl: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StateResponse {
    pub vault_balance: u64,
    pub insurance_fund: u64,
    pub total_capital: u64,
    pub total_pnl: u64,
    pub coverage_ratio_bps: u16,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TradeEvent {
    pub user: Pubkey,
    pub pnl_change: i64,
}

#[event]
pub struct WithdrawalEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[error]
pub enum CustomError {
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Insufficient balance")]
    InsufficientBalance,
}
