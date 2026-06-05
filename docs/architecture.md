# PAPER Protocol — Architecture (Foundation Phase)

This document describes the architecture represented in this repository. It is a
condensed companion to the full Whitepaper and Technical Specification.

## Scope of this repository

This repository implements the **reserve accounting backbone** of PAPER Protocol
as a working Anchor program. It is intentionally minimal: it proves the economic
invariants on-chain without yet implementing the confidential transfer and
governance layers, which depend on infrastructure described below.

## The reserve model

eUSD by Softseco is a fully reserved stablecoin. Every unit issued is backed at
no less than 100% by reserves, held in a documented 20/40/40 allocation:

- **20% liquidity buffer** — USDC in a multi-signature vault, for instant redemption
- **40% tokenized US Treasuries** — regulated short-duration RWA products
- **40% delta-neutral DeFi lending** — stablecoin lending on established protocols

The program enforces two invariants on-chain:

1. **Backing invariant** — total issued eUSD never exceeds total reserve held.
   Every `record_mint` and `record_redeem` re-checks this and fails closed.
2. **Allocation invariant** — `validate_allocation` confirms a proposed reserve
   split matches the 20/40/40 model exactly before reserves are deployed.

## The zero-fee model

PAPER charges no mint or redeem fees. In `record_mint`, the issued amount equals
the deposited amount exactly. Protocol revenue is intended to come from reserve
yield (documented in the Whitepaper), not from user transaction fees.

## Layers planned for later phases

The following are specified in the Technical Specification and are **not** part of
this proof-of-concept:

### Confidential transfers
Token-2022 Confidential Transfers (Twisted ElGamal encryption with Bulletproof
range proofs) provide commercial confidentiality: transfer amounts and balances
are hidden, while the transaction graph remains visible at the Token-2022 level.
This depends on the ZK ElGamal Proof Program, which has been temporarily disabled
on Solana mainnet/devnet since June 2025, with reactivation anticipated in the
Agave 4.x cycle during 2026.

### Identity Whitelist Registry (IWR)
Gates mint and redeem at the compliance perimeter. Circulation between whitelisted
entry and exit points is free and confidential — the same model as physical cash.

### Auditor Key
A multisig-protected ElGamal key enabling selective decryption of specific
transfers under lawful mandate, with every use publicly logged on-chain after a
24-hour delay.

### Shielded Liquidity Vault (SLV)
An AMM with encrypted reserves and zero-knowledge verification of the constant
product invariant, neutralizing MEV (sandwich and front-running attacks).

### Governance
Squads v4 2-of-3 multisig with a 24-hour time-lock on every protocol upgrade.

## Honest note on this proof-of-concept

This program does not implement confidentiality. It implements the accounting and
invariant layer that the confidential layer will sit on top of. The purpose at
Foundation Phase is to demonstrate that the economic model is sound and
implementable on-chain, while the confidential transfer tooling is developed as
the open-source SDK described in the Whitepaper (§9.6).
