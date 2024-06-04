import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { randomBytes } from "crypto";
import assert from "assert";
import {
  getProgramAuthority,
  getTransmuterStruct,
  getTransmuterStructs,
} from "../utils";
import { creator, inputCollection, outputCollection } from "./1_init";
import { program, programId } from "..";

require("dotenv").config({ path: ".env" });

// Random seed
export const seed = new BN(randomBytes(8));

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
    rule: {
      name: "traits",
      rule_type: "match",
      trait_types: [
        ["Background", "red"],
        ["Outfit", "cope"],
      ],
    },
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

it("should add one input to the transmuter", async () => {
  const transmuterStructBefore = await getTransmuterStruct(
    program,
    creator.publicKey,
    seed
  );

  const inputInfo = {
    token_standard: "nft",
    collection: inputCollection.nft.address.toBase58(),
    method: "burn",
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
      title: "Generug split output",
      symbol: "SPLIT",
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

it("should add one output to the transmuter", async () => {
  const outputInfo = {
    token_standard: "nft",
    collection: outputCollection.nft.address.toBase58(),
    method: "mint",
    amount: 1,
    mint: {
      title: "Generug output",
      symbol: "NFT",
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
