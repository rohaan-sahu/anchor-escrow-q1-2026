import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrowQ12026 } from "../target/types/anchor_escrow_q1_2026";
// import { expect } from "chai";
import assert from "node:assert";
//import { describe, before, it } from "node:test";
import { 
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  createMint,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID
 } from "@solana/spl-token";

 import TAKER_ADDRESS from '../keypairs/taker_keypair_1.json';
 import MINT_X_ADDRESS from '../keypairs/mint_x_keypair_1.json';
 import MINT_Y_ADDRESS from '../keypairs/mint_y_keypair_1.json';

describe("anchor_escrow_q1_2026", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorEscrowQ12026 as Program<AnchorEscrowQ12026>;

  const maker = provider.wallet.payer;
  const taker = anchor.web3.Keypair.fromSecretKey(new Uint8Array(TAKER_ADDRESS));

  let mintX: anchor.web3.PublicKey;
  let mintY: anchor.web3.PublicKey;

  let makerAtaX: anchor.web3.PublicKey;
  let takerAtaX: anchor.web3.PublicKey;

  let makerAtaY: anchor.web3.PublicKey;
  let takerAtaY: anchor.web3.PublicKey;

  const seed = new anchor.BN(1234);
  let escrowPda: anchor.web3.PublicKey;
  let escrowBump: number;
  let vault: anchor.web3.PublicKey;

  const depositAmount = 100;
  const receiveAmount = 200;

  before(async () => {
    // Airdrop SOL to maker and taker
    await provider.connection.requestAirdrop(maker.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(taker.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create mints (decimals=0 for simplicity)
    mintX = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      0,
      anchor.web3.Keypair.fromSecretKey(new Uint8Array(MINT_X_ADDRESS))
    );

    mintY = await createMint(
      provider.connection,
      maker,
      taker.publicKey,
      null,
      0,
      anchor.web3.Keypair.fromSecretKey(new Uint8Array(MINT_Y_ADDRESS))
    );

    // Create ATAs and mint tokens
    // makerAtaX = await getOrCreateAssociatedTokenAccount(
    //   provider.connection,
    //   maker,
    //   mintX,
    //   maker.publicKey
    // );

    const newMakerAtaX = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      maker,
      mintX,
      maker.publicKey
    );

    console.log("New ATA:",newMakerAtaX);

    // const makerAtaXTx = new anchor.web3.Transaction().add(
    //   createAssociatedTokenAccountInstruction(
    //     maker.publicKey,
    //     makerAtaX,
    //     maker.publicKey,
    //     mintX
    //   )
    // );
    // await provider.sendAndConfirm(makerAtaXTx);

    await new Promise(resolve => setTimeout(resolve,2000));
    
    await mintTo(
      provider.connection,
      maker,
      mintX,
      newMakerAtaX.address,
      maker,
      depositAmount * 2
    ).then(
      async() => {
        const balanceX = await provider.connection.getTokenAccountBalance(makerAtaX);
        console.log("makerAaX: ",balanceX);
      }
    )

    await new Promise(resolve => setTimeout(resolve,1000));

    takerAtaY = getAssociatedTokenAddressSync(mintY, taker.publicKey);
    const takerAtaYTx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(taker.publicKey, takerAtaY, taker.publicKey, mintY)
    );
    await provider.sendAndConfirm(takerAtaYTx, [taker]);
    await mintTo(provider.connection, taker, mintY, takerAtaY, taker, receiveAmount * 2);


  });

  it("Makes and refunds the escrow", async () => {
    const seed1 = new anchor.BN(1111);
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed1.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintX, escrowPda, true);

    console.log(`
      maker:\t\t${maker.publicKey.toBase58()},
      mint_X:\t\t${mintX.toBase58()},
      mint_Y:\t\t${mintY.toBase58()},
      maker_ATA_X:\t${makerAtaX.toBase58()},
      escrow_pda:\t${escrowPda.toBase58()},
      vault:\t\t${vault.toBase58()}
    }`
    );

    // Make
    await program.methods
      .make(seed1, new anchor.BN(depositAmount), new anchor.BN(receiveAmount))
      .accountsStrict({
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        maker: maker.publicKey,
        mintX: mintX,
        mintY: mintY,
        makerAtaX: makerAtaX,
        escrow: escrowPda,
        vault: vault,
      })
      .signers([maker])
      .rpc();
    

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    assert.strictEqual(escrowAccount.maker.toBase58(),maker.publicKey.toBase58());
    assert.strictEqual(escrowAccount.mintX.toBase58(),mintX.toBase58());
    assert.strictEqual(escrowAccount.mintY.toBase58(),mintY.toBase58());
    assert.strictEqual(escrowAccount.receive.toNumber(),receiveAmount);
    assert.strictEqual(escrowAccount.bump,escrowBump);

    const vaultBalance = (await provider.connection.getTokenAccountBalance(vault)).value.uiAmount;
    assert.strictEqual(vaultBalance,depositAmount);

    // Refund
    /*
    await program.methods
      .refund(seed1)
      .accountsStrict({
        maker: maker.publicKey,
        mintX,
        mintY,
        makerAtaX,
        escrow: escrowPda,
        vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Check closed
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    //assert.strictEqual(escrowInfo,null);

    const vaultInfo = await provider.connection.getAccountInfo(vault);
    //assert.strictEqual(vaultInfo,null);
    */
  });

  /*
  it("Makes and takes the escrow", async () => {
    const seed2 = new anchor.BN(2222);
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintX, escrowPda, true);

    // Make (again for take path)
    await program.methods
      .make(seed2, new anchor.BN(depositAmount), new anchor.BN(receiveAmount))
      .accountsStrict({
        maker: maker.publicKey,
        mintX: mintX,
        mintY: mintY,
        makerAtaX: makerAtaX,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Setup for take
    takerAtaX = getAssociatedTokenAddressSync(mintX, taker.publicKey);
    makerAtaY = getAssociatedTokenAddressSync(mintY, maker.publicKey);

    // Take
    await program.methods
      .take()
      .accountsStrict({
        taker: taker.publicKey,
        maker: maker.publicKey,
        mintX: mintX,
        mintY: mintY,
        takerAtaX: takerAtaX,
        takerAtaY: takerAtaY,
        makerAtaY: makerAtaY,
        makerAtaX: makerAtaX,
        escrow: escrowPda,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([taker])
      .rpc();

    // Check closed
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    //assert.strictEqual(escrowInfo,null);

    const vaultInfo = await provider.connection.getAccountInfo(vault);
    //assert.strictEqual(vaultInfo,null);

    // Check balances
    const takerBalanceA = (await provider.connection.getTokenAccountBalance(takerAtaX)).value.uiAmount;
    //assert.strictEqual(takerBalanceA,depositAmount);

    const makerBalanceB = (await provider.connection.getTokenAccountBalance(makerAtaY)).value.uiAmount;
    //assert.strictEqual(makerBalanceB,receiveAmount);
  });
  */
});
