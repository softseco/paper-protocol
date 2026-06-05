import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PaperProtocol } from "../target/types/paper_protocol";
import { assert } from "chai";

// Integration tests for the PAPER Protocol reserve accounting program.
//
// Unlike a pure arithmetic check, these tests deploy the program to a local
// validator, invoke each instruction as a real Solana transaction, and read
// the resulting on-chain account state back to assert correctness.

describe("paper_protocol", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PaperProtocol as Program<PaperProtocol>;
  const authority = provider.wallet as anchor.Wallet;

  // Derive the reserve PDA (must match seeds in the program).
  const [reservePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reserve")],
    program.programId
  );

  it("initializes the reserve state", async () => {
    await program.methods
      .initializeReserve()
      .accounts({
        authority: authority.publicKey,
      })
      .rpc();

    const reserve = await program.account.reserveState.fetch(reservePda);
    assert.ok(reserve.authority.equals(authority.publicKey));
    assert.equal(reserve.totalReserve.toNumber(), 0);
    assert.equal(reserve.totalIssued.toNumber(), 0);
  });

  it("records a mint and enforces 1:1 zero-fee issuance", async () => {
    const deposit = 1_000_000; // 1,000,000 USDC base units

    await program.methods
      .recordMint(new anchor.BN(deposit))
      .accounts({
        reserveState: reservePda,
        authority: authority.publicKey,
      })
      .rpc();

    const reserve = await program.account.reserveState.fetch(reservePda);
    // Zero-fee model: issued must equal deposit exactly.
    assert.equal(reserve.totalReserve.toNumber(), deposit);
    assert.equal(reserve.totalIssued.toNumber(), deposit);
    // Backing invariant: issued <= reserve.
    assert.ok(reserve.totalIssued.toNumber() <= reserve.totalReserve.toNumber());
  });

  it("records a redemption and keeps the backing invariant", async () => {
    const redeem = 400_000;

    await program.methods
      .recordRedeem(new anchor.BN(redeem))
      .accounts({
        reserveState: reservePda,
        authority: authority.publicKey,
      })
      .rpc();

    const reserve = await program.account.reserveState.fetch(reservePda);
    assert.equal(reserve.totalReserve.toNumber(), 1_000_000 - redeem);
    assert.equal(reserve.totalIssued.toNumber(), 1_000_000 - redeem);
    assert.ok(reserve.totalIssued.toNumber() <= reserve.totalReserve.toNumber());
  });

  it("rejects a redemption larger than outstanding issuance", async () => {
    let failed = false;
    try {
      await program.methods
        .recordRedeem(new anchor.BN(999_999_999))
        .accounts({
          reserveState: reservePda,
          authority: authority.publicKey,
        })
        .rpc();
    } catch (err) {
      failed = true;
    }
    assert.isTrue(failed, "redemption exceeding issuance should fail");
  });

  it("validates the 20/40/40 reserve allocation model", async () => {
    const total = 1_000_000;
    await program.methods
      .validateAllocation(
        new anchor.BN(total),
        new anchor.BN(total / 5), // 20% liquidity buffer
        new anchor.BN((total * 2) / 5), // 40% treasuries
        new anchor.BN((total * 2) / 5) // 40% defi lending
      )
      .accounts({
        authority: authority.publicKey,
      })
      .rpc();
    // If the instruction returns without throwing, the on-chain
    // allocation invariant passed.
    assert.ok(true);
  });

  it("rejects an allocation that breaks the 20/40/40 model", async () => {
    const total = 1_000_000;
    let failed = false;
    try {
      await program.methods
        .validateAllocation(
          new anchor.BN(total),
          new anchor.BN(total / 2), // wrong: 50%
          new anchor.BN(total / 4), // wrong: 25%
          new anchor.BN(total / 4) // wrong: 25%
        )
        .accounts({
          authority: authority.publicKey,
        })
        .rpc();
    } catch (err) {
      failed = true;
    }
    assert.isTrue(failed, "invalid allocation should be rejected on-chain");
  });
});
