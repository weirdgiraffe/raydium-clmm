use crate::states::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token;
use anchor_spl::token::{Mint, Token};
use metaplex_token_metadata::{instruction::create_metadata_accounts, state::Creator};
use spl_token::instruction::AuthorityType;

#[derive(Accounts)]
pub struct PersonalPositionWithMetadata<'info> {
    /// Pays to generate the metadata
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Authority of the NFT mint
    pub amm_config: Account<'info, AmmConfig>,

    /// Mint address for the tokenized position
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,

    /// Position state of the tokenized position
    #[account(
        seeds = [POSITION_SEED.as_bytes(), nft_mint.key().as_ref()],
        bump = position_state.bump
    )]
    pub position_state: Account<'info, PersonalPositionState>,

    /// To store metaplex metadata
    /// CHECK: Safety check performed inside function body
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Sysvar for metadata account creation
    pub rent: Sysvar<'info, Rent>,

    /// Program to create NFT metadata
    /// CHECK: Metadata program address constraint applied
    #[account(address = metaplex_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// Program to update mint authority
    pub token_program: Program<'info, Token>,

    /// Program to allocate lamports to the metadata account
    pub system_program: Program<'info, System>,
}

pub fn personal_position_with_metadata(ctx: Context<PersonalPositionWithMetadata>) -> Result<()> {
    let seeds = [&[ctx.accounts.amm_config.bump] as &[u8]];
    let create_metadata_ix = create_metadata_accounts(
        ctx.accounts.metadata_program.key(),
        ctx.accounts.metadata_account.key(),
        ctx.accounts.nft_mint.key(),
        ctx.accounts.amm_config.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.amm_config.key(),
        String::from("Raydium AMM V3 Positions"),
        String::from(""),
        String::from(""),
        Some(vec![Creator {
            address: ctx.accounts.amm_config.key(),
            verified: true,
            share: 100,
        }]),
        0,
        true,
        false,
    );
    solana_program::program::invoke_signed(
        &create_metadata_ix,
        &[
            ctx.accounts.metadata_account.to_account_info().clone(),
            ctx.accounts.nft_mint.to_account_info().clone(),
            ctx.accounts.payer.to_account_info().clone(),
            ctx.accounts.amm_config.to_account_info().clone(), // mint and update authority
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.to_account_info().clone(),
        ],
        &[&seeds[..]],
    )?;

    // Disable minting
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info().clone(),
            token::SetAuthority {
                current_authority: ctx.accounts.amm_config.to_account_info().clone(),
                account_or_mint: ctx.accounts.nft_mint.to_account_info().clone(),
            },
            &[&seeds[..]],
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    Ok(())
}
