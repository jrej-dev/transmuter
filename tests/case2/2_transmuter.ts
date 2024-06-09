import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { randomBytes } from "crypto";
import { getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import assert from "assert";
import {
  TraitInfo,
  getMetadata,
  getTransmuterStruct,
  getTransmuterStructs,
} from "../utils";
import {
  creator,
  userMetaplex,
  creatorCollection,
  inputCollection,
  outputCollection,
} from "./1_init";
import { program, programId } from "..";
import axios from "axios";

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
  "https://bafkreifhopppitjjmo3dxy2hjdqcodknffp6ink6eagey2iicwnxcvsjqi.ipfs.nftstorage.link";

console.log(`auth: ${auth.toBase58()}`);
console.log(`transmuter2: ${transmuter.toBase58()}`);

it("creates a transformer as a holder", async () => {
  const creatorNfts = await userMetaplex
    .nfts()
    .findAllByOwner({ owner: creator.publicKey });

  const nftsWithCollection = creatorNfts.filter((nft) => nft.collection) as {
    address: PublicKey;
    mintAddress: PublicKey;
    collection: { address: PublicKey };
  }[];

  const foundNft =
    nftsWithCollection.length > 0 &&
    nftsWithCollection[0].collection.address.toBase58() ===
      creatorCollection.nft.address.toBase58();

  let metadata: PublicKey | undefined = undefined;
  let ata: { address?: PublicKey } = {};

  if (foundNft) {
    metadata = await getMetadata(nftsWithCollection[0].mintAddress);
    ata = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      creator,
      nftsWithCollection[0].mintAddress,
      creator.publicKey,
      false
    );
  }

  assert.notEqual(metadata, undefined, "Metadata not found");
  assert.notEqual(ata, undefined, "Ata not found");

  await program.methods
    .transmuterCreateHolder(seed, new BN(1), new BN(8), traitsUri)
    .accounts({
      creator: creator.publicKey,
      auth,
      transmuter: transmuter,
      systemProgram: SystemProgram.programId,
      holderAta: ata.address,
      holderMetadata: metadata,
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

// Wanna test titan dog flow
// split into 5 parts

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

it("should add the dog output to the transmuter", async () => {
  const outputInfo = {
    token_standard: "nft",
    collection: outputCollection.nft.address.toBase58(),
    method: "mint",
    amount: 1,
    rule: {
      name: "split",
      rule_type: "mint",
      trait_types: [
        [await getTraitId(traitsUri, "Dog Breed"), "*"],
        [await getTraitId(traitsUri, "Dog Color"), "*"],
      ],
    },
    mint: {
      title: "Dog pilot",
      symbol: "DPLT",
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

it("should add titan parts output to the transmuter", async () => {
  const titanParts = [
    "Cockpit",
    "Legs",
    "Body",
    "Left Arm",
    "Right Arm",
    "Engine",
  ];

  for (let titanPart of titanParts) {
    const baseColor = await getTraitId(traitsUri, "Base Color");
    const titanPartString = await getTraitId(traitsUri, titanPart);
    const outputInfo = {
      token_standard: "nft",
      collection: outputCollection.nft.address.toBase58(),
      method: "mint",
      amount: 1,
      rule: {
        name: "split",
        rule_type: "mint",
        trait_types: [
          [baseColor, "*"],
          [titanPartString, "*"],
        ],
      },
      mint: {
        title: "Titan part",
        symbol: "TPRT",
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
  }
});

it("should add the color output to the transmuter", async () => {
  const outputInfo = {
    token_standard: "nft",
    collection: outputCollection.nft.address.toBase58(),
    method: "mint",
    amount: 1,
    rule: {
      name: "split",
      rule_type: "mint",
      trait_types: [
        [await getTraitId(traitsUri, "Main Color"), "*"],
        [await getTraitId(traitsUri, "Pattern"), "*"],
      ],
    },
    mint: {
      title: "Titan color",
      symbol: "TCLR",
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

export const getTraitId = async (
  traitsUri: string,
  traitType: string
): Promise<string> => {
  const transmuterTraits = await axios.get(traitsUri).then((res) => res.data);
  const transmuterTrait = transmuterTraits.find(
    (transmuterTrait: TraitInfo) => transmuterTrait.trait_type === traitType
  );
  return transmuterTrait.trait_type_id;
};

export const getValueId = async (
  traitsUri: string,
  value: string
): Promise<string> => {
  const transmuterTraits = await axios.get(traitsUri).then((res) => res.data);
  const transmuterTrait = transmuterTraits.find(
    (transmuterTrait: TraitInfo) => transmuterTrait.value === value
  );
  return transmuterTrait.value_id;
};
