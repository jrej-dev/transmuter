import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { randomBytes } from "crypto";
import assert from "assert";
import {
  getMetadata,
  getProgramAuthority,
  getTransmuterStruct,
  getTransmuterStructs,
} from "../utils";
import {
  creator,
  inputCollection,
  inputMints,
  outputCollection,
  user,
} from "./1_init";
import { program, programId } from "..";
import {
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID as tokenProgram,
} from "@solana/spl-token";

require("dotenv").config({ path: ".env" });

// Random seed
export const seed = new BN(randomBytes(8));
export const vaultSeed = new anchor.BN(randomBytes(8));

const transmuter = PublicKey.findProgramAddressSync(
  [
    Buffer.from("transmuter"),
    creator.publicKey.toBytes(),
    seed.toBuffer().reverse(),
  ],
  program.programId
)[0];

export const auth = PublicKey.findProgramAddressSync(
  [Buffer.from("auth"), transmuter.toBytes()],
  program.programId
)[0];

const traitsUri =
  "https://bafkreiaum2ncnoacx6la6o4anebrvgqgsoqymk62md5vw5mbbt2jvhfzfe.ipfs.nftstorage.link";

console.log(`auth: ${auth.toBase58()}`);
console.log(`transmuter: ${transmuter.toBase58()}`);

it("creates the transmuter", async () => {
  const owner = await getProgramAuthority(
    anchor.getProvider().connection,
    programId
  );
  console.log("owner: ", owner?.toBase58());

  const wba = new PublicKey("3LSY4UTEFt7V7eGsiaAUDzn3iKAJFBPkYseXpdECFknF");
  console.log("wba: ", wba.toBase58());

  await program.methods
    .transmuterCreate(seed, new BN(2), new BN(2), traitsUri)
    .accounts({
      creator: creator.publicKey,
      auth,
      transmuter,
      systemProgram: SystemProgram.programId,
      owner,
      wba,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });
});

it("checks one transmuter has been created", async () => {
  const transmuters = await getTransmuterStructs(program, creator.publicKey);
  assert.equal(transmuters.length, 1);
});

it("should add one input to the transmuter", async () => {
  const transmuterStructBefore = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const inputInfo = {
    token_standard: "nft",
    collection: inputCollection.nft.address.toBase58(),
    method: "transfer",
    amount: 1,
  };

  await program.methods
    .transmuterSetInput(seed, JSON.stringify(inputInfo))
    .accounts({
      creator: creator.publicKey,
      transmuter,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });

  const transmuterStructAfter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  assert.equal(
    transmuterStructAfter.account.inputs.length,
    transmuterStructBefore.account.inputs.length + 1
  );
  assert.deepEqual(
    JSON.parse(transmuterStructAfter.account.inputs.slice(-1)[0]),
    inputInfo
  );
});

it("should add one output to the transmuter", async () => {
  const outputInfo = {
    token_standard: "nft",
    collection: outputCollection.nft.address.toBase58(),
    method: "mint",
    amount: 1,
    mint: {
      title: "Generug output",
      symbol: "GNRG",
      uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
    },
  };

  await program.methods
    .transmuterSetOutput(seed, JSON.stringify(outputInfo))
    .accounts({
      creator: creator.publicKey,
      transmuter,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });
});

it("should close an existing transmuter", async () => {
  await program.methods
    .transmuterClose()
    .accounts({
      creator: creator.publicKey,
      transmuter,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });
});

it("checks that no transmuter are created", async () => {
  const transmuters = await getTransmuterStructs(program, creator.publicKey);
  assert.equal(transmuters.length, 0);
});

it("should fail to handle input", async () => {
  try {
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

    console.log("vaultAuth: ", vaultAuth.toBase58());

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
  } catch (e) {
    assert.ok(e instanceof Error);
    return;
  }
  assert.fail("Test should have failed");
});

it("creates a new transmuter", async () => {
  const owner = await getProgramAuthority(
    anchor.getProvider().connection,
    programId
  );
  console.log("owner: ", owner?.toBase58());

  const wba = new PublicKey("3LSY4UTEFt7V7eGsiaAUDzn3iKAJFBPkYseXpdECFknF");
  console.log("wba: ", wba.toBase58());

  await program.methods
    .transmuterCreate(seed, new BN(2), new BN(2), traitsUri)
    .accounts({
      creator: creator.publicKey,
      auth,
      transmuter,
      systemProgram: SystemProgram.programId,
      owner,
      wba,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });
});

it("checks the transmuter has been created", async () => {
  const transmuter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );
  assert.ok(transmuter);
});

it("should add one input to the transmuter", async () => {
  const transmuterStructBefore = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const inputInfo = {
    token_standard: "nft",
    collection: inputCollection.nft.address.toBase58(),
    method: "transfer",
    amount: 1,
  };

  await program.methods
    .transmuterSetInput(seed, JSON.stringify(inputInfo))
    .accounts({
      creator: creator.publicKey,
      transmuter,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });

  const transmuterStructAfter = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  assert.equal(
    transmuterStructAfter.account.inputs.length,
    transmuterStructBefore.account.inputs.length + 1
  );
  assert.deepEqual(
    JSON.parse(transmuterStructAfter.account.inputs.slice(-1)[0]),
    inputInfo
  );
});

it("should add one output to the transmuter", async () => {
  const outputInfo = {
    token_standard: "nft",
    collection: outputCollection.nft.address.toBase58(),
    method: "mint",
    amount: 1,
    mint: {
      title: "Generug output",
      symbol: "GNRG",
      uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
    },
  };

  await program.methods
    .transmuterSetOutput(seed, JSON.stringify(outputInfo))
    .accounts({
      creator: creator.publicKey,
      transmuter,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });
});

it("should pause the transmuter", async () => {
  await program.methods
    .transmuterPause(seed)
    .accounts({
      creator: creator.publicKey,
      transmuter,
    })
    .signers([creator])
    .rpc({
      skipPreflight: true,
    });

  const transmuterStruct = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  assert.ok(transmuterStruct.account.locked);
});
