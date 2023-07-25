use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount, Transfer},
};

use super::*;

#[derive(Accounts)]
pub struct InitializeMetaDAO<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + (100 * 32), // 100 member max
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"], // abbreviation of the last two sentences of the Declaration of Independence of Cyberspace
        bump
    )]
    pub meta_dao: Account<'info, MetaDAO>,
    pub seed_member: Account<'info, Member>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitializeMember<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + 20 + 1 + 32,
        seeds = [b"member", name.as_bytes()], // 256^20 possible names, so practically impossible for all names to be exhausted
        bump
    )]
    pub member: Account<'info, Member>,
    #[account(
        init,
        payer = initializer,
        space = 8,
        seeds = [b"treasury", member.key().as_ref()],
        bump
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub treasury: UncheckedAccount<'info>,
    #[account(
        init,
        payer = initializer,
        mint::authority = member,
        mint::freeze_authority = member,
        mint::decimals = 9
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(
        signer @ ErrorCode::UnauthorizedFunctionCall,
        mut,
        seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"], 
        bump
    )]
    pub meta_dao: Account<'info, MetaDAO>,
    pub member: Account<'info, Member>,
}

#[derive(Accounts)]
pub struct InitializeProposal<'info> {
    #[account(zero)]
    pub proposal: Account<'info, Proposal>,
    #[account(seeds = [b"WWCACOTMICMIBMHAFTTWYGHMB"], bump)]
    pub meta_dao: Account<'info, MetaDAO>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct FailProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
#[instruction(pass_or_fail: PassOrFail)]
pub struct InitializeConditionalExpression<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + 32 + 1,
        seeds = [
            b"conditional_expression", 
            proposal.key().as_ref(),
            &[pass_or_fail as u8]
        ],
        bump
    )]
    pub conditional_expression: Account<'info, ConditionalExpression>,
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + (32 * 4) + 1,
        seeds = [
            b"vault", 
            conditional_expression.key().as_ref(),
            underlying_token_mint.key().as_ref()
        ],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub conditional_expression: Account<'info, ConditionalExpression>,
    /// SPL mint of the underlying token
    pub underlying_token_mint: Account<'info, Mint>,
    /// token account for the vault that matches above mint
    #[account(
        init,
        payer = initializer,
        associated_token::authority = vault,
        associated_token::mint = underlying_token_mint
    )]
    pub vault_underlying_token_account: Account<'info, TokenAccount>,
    /// SPL mint of the conditional token
    #[account(
        init,
        payer = initializer,
        mint::authority = vault,
        mint::freeze_authority = vault,
        mint::decimals = underlying_token_mint.decimals
    )]
    pub conditional_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct InitializeDepositSlip<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + 32 + 32 + 8,
        seeds = [b"deposit_slip", vault.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub deposit_slip: Account<'info, DepositSlip>,
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintConditionalTokens<'info> {
    #[account(
        has_one = conditional_token_mint @ ErrorCode::InvalidConditionalTokenMint,
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub conditional_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = vault_underlying_token_account.key() == vault.underlying_token_account @  ErrorCode::InvalidVaultUnderlyingTokenAccount
    )]
    pub vault_underlying_token_account: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    #[account(
        mut,
        has_one = user,
        has_one = vault
    )]
    pub deposit_slip: Account<'info, DepositSlip>,
    #[account(
        mut,
        token::authority = user,
        token::mint = vault.underlying_token_mint,
        constraint = user_underlying_token_account.amount >= amount @ ErrorCode::InsufficientUnderlyingTokens
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = user,
        token::mint = conditional_token_mint
    )]
    pub user_conditional_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> MintConditionalTokens<'info> {
    pub fn into_transfer_underlying_tokens_to_vault_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.user_underlying_token_account.to_account_info().clone(),
            to: self
                .vault_underlying_token_account
                .to_account_info()
                .clone(),
            authority: self.user.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }

    pub fn into_mint_conditional_tokens_to_user_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.conditional_token_mint.to_account_info().clone(),
            to: self
                .user_conditional_token_account
                .to_account_info()
                .clone(),
            authority: self.vault.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct RedeemConditionalTokensForUnderlyingTokens<'info> {
    #[account(
        has_one = conditional_expression @ ErrorCode::InvalidConditionalExpression,
        has_one = conditional_token_mint @ ErrorCode::InvalidConditionalTokenMint
    )]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub conditional_token_mint: Account<'info, Mint>,
    #[account(has_one = proposal @ ErrorCode::InvalidProposal)]
    pub conditional_expression: Account<'info, ConditionalExpression>,
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        constraint = vault_underlying_token_account.key() == vault.underlying_token_account @ ErrorCode::InvalidVaultUnderlyingTokenAccount
    )]
    pub vault_underlying_token_account: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    #[account(
        mut,
        token::authority = user,
        token::mint = conditional_token_mint
    )]
    pub user_conditional_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = user,
        token::mint = vault.underlying_token_mint
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RedeemConditionalTokensForUnderlyingTokens<'info> {
    pub fn into_burn_conditional_tokens_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.conditional_token_mint.to_account_info().clone(),
            from: self
                .user_conditional_token_account
                .to_account_info()
                .clone(),
            authority: self.user.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }

    pub fn into_transfer_underlying_tokens_to_user_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .vault_underlying_token_account
                .to_account_info()
                .clone(),
            to: self.user_underlying_token_account.to_account_info().clone(),
            authority: self.vault.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct RedeemDepositSlipForUnderlyingTokens<'info> {
    #[account(
        has_one = conditional_expression @ ErrorCode::InvalidConditionalExpression,
    )]
    pub vault: Account<'info, Vault>,
    #[account(has_one = proposal @ ErrorCode::InvalidProposal)]
    pub conditional_expression: Account<'info, ConditionalExpression>,
    pub proposal: Account<'info, Proposal>,
    #[account(mut, constraint = vault_underlying_token_account.key() == vault.underlying_token_account @ ErrorCode::InvalidVaultUnderlyingTokenAccount)]
    pub vault_underlying_token_account: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
    #[account(
        mut, 
        has_one = user, 
        has_one = vault
    )]
    pub user_deposit_slip: Account<'info, DepositSlip>,
    #[account(
        mut,
        token::authority = user,
        token::mint = vault_underlying_token_account.mint
    )]
    pub user_underlying_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RedeemDepositSlipForUnderlyingTokens<'info> {
    pub fn into_transfer_underlying_tokens_to_user_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .vault_underlying_token_account
                .to_account_info()
                .clone(),
            to: self.user_underlying_token_account.to_account_info().clone(),
            authority: self.vault.to_account_info().clone(),
        };
        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}
