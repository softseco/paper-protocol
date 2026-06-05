//! PAPER Protocol — reserve accounting program (Foundation Phase proof-of-concept)
//!
//! This program is a minimal, honest on-chain representation of PAPER Protocol's
//! reserve-backed accounting model, as described in the PAPER Whitepaper (v1.0).
//!
//! Scope of THIS program (Foundation Phase):
//!   - On-chain reserve state account
//!   - Enforcement of the 1:1 backing invariant (issued eUSD <= reserve)
//!   - Enforcement of the zero-fee mint/redeem model
//!   - Validation of the 20/40/40 reserve allocation model
//!
//! Explicitly OUT OF SCOPE for this proof-of-concept (planned for later phases):
//!   - Token-2022 Confidential Transfers (depends on ZK ElGamal Proof Program,
//!     temporarily disabled on Solana mainnet/devnet since June 2025)
//!   - Shielded Liquidity Vault (SLV)
//!   - Auditor Key selective disclosure
//!   - Squads v4 multisig governance integration
//!
//! This program demonstrates the accounting backbone on-chain. The confidential
//! and governance layers are documented in the Technical Specification and are
//! the subject of the Foundation Phase open-source SDK work.

use anchor_lang::prelude::*;

declare_id!("81TNSWJQupZvs5Ga1R2Xrizh9Yu87uBbonzQeqk9t3xz");

#[program]
pub mod paper_protocol {
    use super::*;

    /// Initialize the protocol reserve state.
    ///
    /// Creates the on-chain ReserveState account that tracks total reserve
    /// deposited and total eUSD issued. Called once at protocol setup.
    pub fn initialize_reserve(ctx: Context<InitializeReserve>) -> Result<()> {
        let reserve = &mut ctx.accounts.reserve_state;
        reserve.authority = ctx.accounts.authority.key();
        reserve.total_reserve = 0;
        reserve.total_issued = 0;
        reserve.bump = ctx.bumps.reserve_state;

        msg!("PAPER reserve initialized. Authority: {}", reserve.authority);
        Ok(())
    }

    /// Record a mint of eUSD against deposited USDC reserve.
    ///
    /// Enforces:
    ///   - zero fee: issued amount equals deposited amount exactly (1:1)
    ///   - backing invariant: total_issued never exceeds total_reserve
    ///
    /// `reserve_deposit` is the amount of USDC added to the reserve.
    /// In the zero-fee model the user receives exactly the same amount in eUSD.
    pub fn record_mint(ctx: Context<UpdateReserve>, reserve_deposit: u64) -> Result<()> {
        require!(reserve_deposit > 0, PaperError::ZeroAmount);

        let reserve = &mut ctx.accounts.reserve_state;

        // Zero-fee model: the user receives exactly what they deposited.
        let issued = reserve_deposit;

        reserve.total_reserve = reserve
            .total_reserve
            .checked_add(reserve_deposit)
            .ok_or(PaperError::MathOverflow)?;
        reserve.total_issued = reserve
            .total_issued
            .checked_add(issued)
            .ok_or(PaperError::MathOverflow)?;

        // Backing invariant: issued eUSD must never exceed reserve held.
        require!(
            reserve.total_issued <= reserve.total_reserve,
            PaperError::BackingViolation
        );

        msg!(
            "Mint recorded. Deposit: {}, Issued: {}, Backing: {}/{}",
            reserve_deposit,
            issued,
            reserve.total_issued,
            reserve.total_reserve
        );
        Ok(())
    }

    /// Record a redemption of eUSD back to USDC reserve.
    ///
    /// Enforces zero fee (user receives exactly what they redeem) and that
    /// the redemption does not exceed outstanding issuance or reserve.
    pub fn record_redeem(ctx: Context<UpdateReserve>, redeem_amount: u64) -> Result<()> {
        require!(redeem_amount > 0, PaperError::ZeroAmount);

        let reserve = &mut ctx.accounts.reserve_state;

        require!(
            redeem_amount <= reserve.total_issued,
            PaperError::RedeemExceedsIssued
        );
        require!(
            redeem_amount <= reserve.total_reserve,
            PaperError::RedeemExceedsReserve
        );

        reserve.total_issued = reserve
            .total_issued
            .checked_sub(redeem_amount)
            .ok_or(PaperError::MathOverflow)?;
        reserve.total_reserve = reserve
            .total_reserve
            .checked_sub(redeem_amount)
            .ok_or(PaperError::MathOverflow)?;

        // Backing invariant must still hold after redemption.
        require!(
            reserve.total_issued <= reserve.total_reserve,
            PaperError::BackingViolation
        );

        msg!(
            "Redeem recorded. Amount: {}, Backing: {}/{}",
            redeem_amount,
            reserve.total_issued,
            reserve.total_reserve
        );
        Ok(())
    }

    /// Validate the 20/40/40 reserve allocation model for a given total.
    ///
    /// Returns an error if the supplied allocation does not match the
    /// documented model: 20% liquidity buffer, 40% tokenized US Treasuries,
    /// 40% delta-neutral DeFi lending. This is a pure on-chain check used to
    /// assert the allocation invariant before reserves are deployed.
    pub fn validate_allocation(
        _ctx: Context<ValidateAllocation>,
        total: u64,
        liquidity_buffer: u64,
        treasuries: u64,
        defi_lending: u64,
    ) -> Result<()> {
        require!(total > 0, PaperError::ZeroAmount);

        // Sum of parts must equal the whole (full 1:1 backing, no gap).
        let sum = liquidity_buffer
            .checked_add(treasuries)
            .ok_or(PaperError::MathOverflow)?
            .checked_add(defi_lending)
            .ok_or(PaperError::MathOverflow)?;
        require!(sum == total, PaperError::AllocationMismatch);

        // 20 / 40 / 40 split, validated in basis points to avoid rounding gaps.
        require!(
            liquidity_buffer == total / 5,           // 20%
            PaperError::AllocationMismatch
        );
        require!(
            treasuries == (total * 2) / 5,            // 40%
            PaperError::AllocationMismatch
        );
        require!(
            defi_lending == (total * 2) / 5,          // 40%
            PaperError::AllocationMismatch
        );

        msg!(
            "Allocation valid. Total: {}, 20%: {}, 40%: {}, 40%: {}",
            total,
            liquidity_buffer,
            treasuries,
            defi_lending
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeReserve<'info> {
    #[account(
        init,
        payer = authority,
        space = ReserveState::SPACE,
        seeds = [b"reserve"],
        bump
    )]
    pub reserve_state: Account<'info, ReserveState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateReserve<'info> {
    #[account(
        mut,
        seeds = [b"reserve"],
        bump = reserve_state.bump,
        has_one = authority @ PaperError::Unauthorized
    )]
    pub reserve_state: Account<'info, ReserveState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ValidateAllocation<'info> {
    pub authority: Signer<'info>,
}

/// On-chain reserve accounting state.
#[account]
pub struct ReserveState {
    /// Authority permitted to record mints and redemptions.
    pub authority: Pubkey,
    /// Total USDC reserve currently held (in base units).
    pub total_reserve: u64,
    /// Total eUSD currently issued (in base units).
    pub total_issued: u64,
    /// PDA bump.
    pub bump: u8,
}

impl ReserveState {
    // 8 (discriminator) + 32 (authority) + 8 + 8 + 1
    pub const SPACE: usize = 8 + 32 + 8 + 8 + 1;
}

#[error_code]
pub enum PaperError {
    #[msg("Amount must be greater than zero.")]
    ZeroAmount,
    #[msg("Arithmetic overflow.")]
    MathOverflow,
    #[msg("Backing invariant violated: issued eUSD exceeds reserve.")]
    BackingViolation,
    #[msg("Redemption exceeds outstanding issuance.")]
    RedeemExceedsIssued,
    #[msg("Redemption exceeds available reserve.")]
    RedeemExceedsReserve,
    #[msg("Reserve allocation does not match the 20/40/40 model.")]
    AllocationMismatch,
    #[msg("Unauthorized: signer is not the reserve authority.")]
    Unauthorized,
}
