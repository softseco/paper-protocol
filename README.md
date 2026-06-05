# PAPER Protocol

**A confidential stablecoin on Solana, designed for issuance under the MiCA E-Money Token framework from Poland, European Union.**

PAPER Protocol is the infrastructure layer behind **eUSD by Softseco** — a confidential, fully reserved stablecoin built on Solana's Token-2022 standard. This repository contains the Foundation Phase proof-of-concept program: the on-chain reserve accounting backbone that enforces the protocol's core economic invariants.

> **Status:** Foundation Phase. Mainnet beta targeted for Q1–Q2 2029. eUSD by Softseco is not yet issued. Softseco sp. z o.o. is to be incorporated.

---

## What this repository contains

This is a **minimal, honest** on-chain implementation of PAPER's reserve accounting model. It demonstrates the protocol's economic backbone as a working Anchor program with real integration tests against a local validator.

**Implemented here (Foundation Phase):**

- On-chain reserve state account (`ReserveState`)
- 1:1 backing invariant enforcement (issued eUSD never exceeds reserve)
- Zero-fee mint/redeem model enforcement
- 20/40/40 reserve allocation model validation

**Documented, planned for later phases (not in this proof-of-concept):**

- Token-2022 Confidential Transfers — Twisted ElGamal + Bulletproofs (depends on the ZK ElGamal Proof Program, temporarily disabled on Solana mainnet/devnet since June 2025, reactivation anticipated in the Agave 4.x cycle during 2026)
- Shielded Liquidity Vault (SLV) for MEV-resistant settlement
- Auditor Key for selective regulatory disclosure
- Squads v4 multisig governance with 24-hour time-lock

These layers are specified in the Technical Specification and are the subject of the Foundation Phase open-source SDK work.

---

## Architecture overview

PAPER eUSD by Softseco is backed 1:1 by a reserve held in a documented allocation:

| Tranche | Share | Instruments |
|---|---|---|
| Liquidity buffer | 20% | USDC in a multi-signature vault |
| Tokenized US Treasuries | 40% | Regulated short-duration RWA (e.g. BUIDL, Ondo) |
| Delta-neutral DeFi lending | 40% | Stablecoin lending on established protocols (Aave, Kamino) |

The protocol charges **zero fees** on mint and redeem. Revenue is generated from reserve yield, not user friction. See the Whitepaper for the full economic model.

---

## Program instructions

| Instruction | Purpose |
|---|---|
| `initialize_reserve` | Create the on-chain reserve state account |
| `record_mint` | Record a mint of eUSD against deposited reserve (enforces 1:1, zero fee) |
| `record_redeem` | Record a redemption (enforces backing invariant) |
| `validate_allocation` | Validate a 20/40/40 reserve allocation on-chain |

All instructions enforce their invariants on-chain and fail closed: any operation that would break the backing invariant or the allocation model is rejected.

---

## Getting started

### Prerequisites

- Rust 1.79+
- Solana CLI 1.18+ (or Agave)
- Anchor 0.31.0
- Node.js 18+
- Yarn

### Build

```bash
yarn install
anchor build
```

### Test

Runs the full integration suite against a local validator:

```bash
anchor test
```

The tests deploy the program, invoke each instruction as a real transaction, and assert on the resulting on-chain account state — including negative tests that confirm invalid operations are rejected.

---

## Repository structure

```
paper_protocol/
├── programs/paper_protocol/   Anchor program (reserve accounting)
│   └── src/lib.rs
├── tests/                     Integration tests (local validator)
│   └── paper_protocol.ts
├── docs/                      Architecture documentation
│   └── architecture.md
├── .github/workflows/         CI (build + test on push)
└── Anchor.toml
```

---

## Documentation

- **Whitepaper** — full protocol, economic model, compliance framework
- **Technical Specification** — engineering-grade detail (available on request)
- **Website** — [project site]

---

## License

Released under the [MIT License](./LICENSE). PAPER Protocol's open-source components are intended to serve the broader Solana ecosystem, not only eUSD by Softseco.

---

## Disclaimer

This repository is published during the Foundation Phase, prior to incorporation of the issuing entity and prior to any regulatory authorization. It does not constitute an offer to sell securities or tokens, and is not investment advice. eUSD by Softseco is not yet issued.
