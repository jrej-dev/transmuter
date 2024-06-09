import assert from "assert";
import * as anchor from "@coral-xyz/anchor";
import { creator, inputMints, user, userMetaplex } from "./1_init";
import { getMetadata, getTransmuterStruct, getvaultAuthStruct } from "../utils";
import {
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID as tokenProgram,
} from "@solana/spl-token";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { seed, vaultSeed } from "./2_transmuter";
import { Metadata } from "@metaplex-foundation/js";
import { program } from "..";

it("should fail for a user to close a transmuter", async () => {
  try {
    //Must have creator and seed to find transmuter
    const transmuter = await getTransmuterStruct(
      program,
      creator.publicKey,
      seed
    );

    await program.methods
      .transmuterClose()
      .accounts({
        creator: user.publicKey,
        transmuter: transmuter.publicKey,
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

it("should init vault auth", async () => {
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

  console.log("vaultAuth: ", vaultAuth.toBase58());

  await program.methods
    .userInitVaultAuth(seed, vaultSeed)
    .accounts({
      creator: creator.publicKey,
      user: user.publicKey,
      vaultAuth,
      transmuter: transmuter.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([user])
    .rpc({
      skipPreflight: true,
    });
});

it("should handle input", async () => {
  const inputMint = inputMints[0].nft.address;

  //Must have creator and seed to find transmuter
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
    vaultAuth.publicKey,
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
      vaultAuth: vaultAuth.publicKey,
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

it("should cancel input", async () => {
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

  const vaultAuthNfts = await userMetaplex.nfts().findAllByOwner({
    owner: vaultAuth.publicKey,
  });

  for (let vaultAuthNft of vaultAuthNfts) {
    const nftMintAddress = (vaultAuthNft as Metadata).mintAddress;

    const ata = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      nftMintAddress,
      user.publicKey,
      true
    );

    const vault = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      nftMintAddress,
      vaultAuth.publicKey,
      true
    );

    await program.methods
      .userCancelInput(seed, vaultSeed)
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        mint: nftMintAddress,
        ata: ata.address,
        vaultAuth: vaultAuth.publicKey,
        vault: vault.address,
        tokenProgram,
        transmuter: transmuter.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc({
        skipPreflight: true,
      });
  }
});

it("should check that there are no inputs", async () => {
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
    vaultAuth.account.handledInputs.filter((input) => input),
    0
  );
});
