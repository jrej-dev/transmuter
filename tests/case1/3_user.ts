import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import { creator, inputMints, user, userMetaplex } from "./1_init";
import {
  TOKEN_METADATA_PROGRAM_ID,
  addPriorityFee,
  confirmTx,
  confirmTxs,
  getMasterEdition,
  getMetadata,
  getTransmuterStruct,
  getvaultAuthStruct,
  modifyComputeUnits,
} from "../utils";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID as associatedTokenProgram,
  createMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID as tokenProgram,
} from "@solana/spl-token";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { auth, seed } from "./2_transmuter";
import { randomBytes } from "crypto";
import { Metadata } from "@metaplex-foundation/js";
import { program } from "..";

export const vaultSeed = new anchor.BN(randomBytes(8));

it("should handle input", async () => {
  const inputMint = inputMints[0].nft.address;

  //Must have creator and seed to find transmuter
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const vaultAuth = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vaultAuth"),
      transmuter.publicKey.toBytes(),
      user.publicKey.toBytes(),
      vaultSeed.toBuffer().reverse(),
    ],
    program.programId
  )[0];

  const ata = await getOrCreateAssociatedTokenAccount(
    anchor.getProvider().connection,
    user,
    inputMint,
    user.publicKey,
    true
  );

  const vault = await getOrCreateAssociatedTokenAccount(
    anchor.getProvider().connection,
    user,
    inputMint,
    vaultAuth,
    true
  );

  const metadata = await getMetadata(inputMint);

  await program.methods
    .userSendInput(seed, vaultSeed)
    .accounts({
      creator: creator.publicKey,
      user: user.publicKey,
      mint: inputMint,
      ata: ata.address,
      metadata: metadata,
      vaultAuth,
      vault: vault.address,
      tokenProgram,
      transmuter: transmuter.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc({
      skipPreflight: true,
    });
});

it("Should fail to handle input as trait is not matching", async () => {
  try {
    const inputMint = inputMints[1].nft.address;

    //Must have creator and seed to find transmuter
    const transmuter = await getTransmuterStruct(
      program,
      creator.publicKey,
      seed
    );

    const vaultAuth = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vaultAuth"),
        transmuter.publicKey.toBytes(),
        user.publicKey.toBytes(),
        vaultSeed.toBuffer().reverse(),
      ],
      program.programId
    )[0];

    const ata = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      inputMint,
      user.publicKey,
      true
    );

    const vault = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      inputMint,
      vaultAuth,
      true
    );

    const metadata = await getMetadata(inputMint);

    await program.methods
      .userSendInput(seed, vaultSeed)
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        mint: inputMint,
        ata: ata.address,
        metadata: metadata,
        vaultAuth: vaultAuth,
        vault: vault.address,
        tokenProgram,
        transmuter: transmuter.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc({
        skipPreflight: true,
      });
  } catch (e) {
    assert.ok(e instanceof Error);
    return;
  }
  assert.fail("Test should have failed");
});

it("should fail to claim output", async () => {
  try {
    //Must have creator and seed to find transmuter
    const transmuter = await getTransmuterStruct(
      program,
      creator.publicKey,
      seed
    );

    const vaultAuth = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vaultAuth"),
        transmuter.publicKey.toBytes(),
        user.publicKey.toBytes(),
        vaultSeed.toBuffer().reverse(),
      ],
      program.programId
    )[0];

    let mint = await createMint(
      anchor.getProvider().connection,
      user,
      auth,
      auth,
      0
    );

    const metadata = await getMetadata(mint);

    const ata = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      mint,
      user.publicKey,
      true
    );

    const masterEdition = await getMasterEdition(mint);

    await program.methods
      .userClaimOutput(seed, vaultSeed)
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        vaultAuth,
        auth,
        transmuter: transmuter.publicKey,
        mint,
        ata: ata.address,
        metadata,
        masterEdition,
        tokenProgram,
        associatedTokenProgram,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([user])
      .rpc({
        skipPreflight: true,
      });
  } catch (e) {
    assert.ok(e instanceof Error);
    return;
  }
  assert.fail("Test should have failed");
});

it("should verify transmuter is not claimable", async () => {
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const vaultAuth = await getvaultAuthStruct(
    program,
    transmuter.publicKey,
    user.publicKey,
    vaultSeed
  );

  assert.notEqual(
    transmuter.account.inputs.length,
    vaultAuth.account.handledInputs.filter((input) => input).length
  );
});

it("should handle input", async () => {
  const inputMint = inputMints[2].nft.address;

  //Must have creator and seed to find transmuter
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const vaultAuth = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vaultAuth"),
      transmuter.publicKey.toBytes(),
      user.publicKey.toBytes(),
      vaultSeed.toBuffer().reverse(),
    ],
    program.programId
  )[0];

  const ata = await getOrCreateAssociatedTokenAccount(
    anchor.getProvider().connection,
    user,
    inputMint,
    user.publicKey,
    true
  );

  const vault = await getOrCreateAssociatedTokenAccount(
    anchor.getProvider().connection,
    user,
    inputMint,
    vaultAuth,
    true
  );

  const metadata = await getMetadata(inputMint);

  await program.methods
    .userSendInput(seed, vaultSeed)
    .accounts({
      creator: creator.publicKey,
      user: user.publicKey,
      mint: inputMint,
      ata: ata.address,
      metadata: metadata,
      vaultAuth,
      vault: vault.address,
      tokenProgram,
      transmuter: transmuter.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc({
      skipPreflight: true,
    });
});

it("should verify transmuter is claimable", async () => {
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const vaultAuth = await getvaultAuthStruct(
    program,
    transmuter.publicKey,
    user.publicKey,
    vaultSeed
  );

  assert.equal(
    transmuter.account.inputs.length,
    vaultAuth.account.handledInputs.filter((input) => input).length
  );
});

it("should claim output", async () => {
  //Must have creator and seed to find transmuter
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  //Must have user and vaultSeed to find vaultAuth
  const vaultAuth = await getvaultAuthStruct(
    program,
    transmuter.publicKey,
    user.publicKey,
    vaultSeed
  );

  let mint = await createMint(
    anchor.getProvider().connection,
    user,
    auth,
    auth,
    0
  );

  const metadata = await getMetadata(mint);

  const ata = await getOrCreateAssociatedTokenAccount(
    anchor.getProvider().connection,
    user,
    mint,
    user.publicKey,
    true
  );

  const masterEdition = await getMasterEdition(mint);

  await program.methods
    .userClaimOutput(seed, vaultSeed)
    .accounts({
      creator: creator.publicKey,
      user: user.publicKey,
      vaultAuth: vaultAuth.publicKey,
      auth,
      transmuter: transmuter.publicKey,
      mint,
      ata: ata.address,
      metadata,
      masterEdition,
      tokenProgram,
      associatedTokenProgram,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .preInstructions([modifyComputeUnits])
    .signers([user])
    .rpc({
      skipPreflight: true,
    })
    .then(confirmTx);
});

it("should verify that vault is locked for user but not complete", async () => {
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const vaultAuth = await getvaultAuthStruct(
    program,
    transmuter.publicKey,
    user.publicKey,
    vaultSeed
  );

  const handledOutputs = vaultAuth.account.handledOutputs.filter(
    (output) => output
  );

  assert.equal(vaultAuth.account.userLock, true);
  assert.equal(vaultAuth.account.creatorLock, false);
  assert.notEqual(transmuter.account.outputs.length, handledOutputs.length);
});

it("should claim output", async () => {
  //Must have creator and seed to find transmuter
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  //Must have user and vaultSeed to find vaultAuth
  const vaultAuth = await getvaultAuthStruct(
    program,
    transmuter.publicKey,
    user.publicKey,
    vaultSeed
  );

  let mint = await createMint(
    anchor.getProvider().connection,
    user,
    auth,
    auth,
    0
  );

  const metadata = await getMetadata(mint);

  const ata = await getOrCreateAssociatedTokenAccount(
    anchor.getProvider().connection,
    user,
    mint,
    user.publicKey,
    true
  );

  const masterEdition = await getMasterEdition(mint);

  await program.methods
    .userClaimOutput(seed, vaultSeed)
    .accounts({
      creator: creator.publicKey,
      user: user.publicKey,
      vaultAuth: vaultAuth.publicKey,
      auth,
      transmuter: transmuter.publicKey,
      mint,
      ata: ata.address,
      metadata,
      masterEdition,
      tokenProgram,
      associatedTokenProgram,
      tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .preInstructions([modifyComputeUnits])
    .signers([user])
    .rpc({
      skipPreflight: true,
    })
    .then(confirmTx);
});

it("should verify that vault is locked for user but not complete", async () => {
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const vaultAuth = await getvaultAuthStruct(
    program,
    transmuter.publicKey,
    user.publicKey,
    vaultSeed
  );

  const handledOutputs = vaultAuth.account.handledOutputs.filter(
    (output) => output
  );

  assert.equal(vaultAuth.account.userLock, true);
  assert.equal(vaultAuth.account.creatorLock, false);
  assert.equal(transmuter.account.outputs.length, handledOutputs.length);
});
