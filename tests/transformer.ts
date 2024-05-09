import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { Transformer, IDL } from "../target/types/transformer";
import {
  PublicKey,
  Commitment,
  Keypair,
  SystemProgram,
  Connection,
} from "@solana/web3.js";
import { randomBytes } from "crypto";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID as associatedTokenProgram,
  createMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID as tokenProgram,
} from "@solana/spl-token";
import {
  Metaplex,
  keypairIdentity,
  mockStorage,
} from "@metaplex-foundation/js";
import axios from "axios";
import sharp from "sharp";
import wallet from "../../../.config/solana/id.json";
import { NFTStorage, File } from "nft.storage";
require("dotenv").config({ path: ".env" });
import { InputInfo, OutputInfo, TraitInfo } from "./types";
import { traits } from "./data";
import assert from "assert";

const commitment: Commitment = "confirmed";

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

const storageClient = new NFTStorage({
  token: process.env.NFT_STORAGE_KEY,
});

describe("transformer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const programId = new PublicKey(
    "H8SJKV7T4egtcwoA2HqSCNYeqrTJuA7SDSeZNrAgMmpf"
  );
  const program = new anchor.Program<Transformer>(
    IDL,
    programId,
    anchor.getProvider()
  );

  // Set up our keys
  const [creator, user, candyMachine] = [
    new Keypair(),
    new Keypair(),
    new Keypair(),
    new Keypair(),
  ];

  const devnetKeypair = anchor.web3.Keypair.fromSecretKey(
    new Uint8Array(wallet)
  );

  console.log(`creator: ${creator.publicKey}`);
  console.log(`user: ${user.publicKey}`);
  console.log(`candyMachine: ${candyMachine.publicKey}`);

  // Random seed
  const seed = new BN(randomBytes(8));

  //create the transumter account
  const auth = PublicKey.findProgramAddressSync(
    [Buffer.from("auth")],
    program.programId
  )[0];

  const transmuter = PublicKey.findProgramAddressSync(
    [
      Buffer.from("transmuter"),
      creator.publicKey.toBytes(),
      seed.toBuffer().reverse(),
    ],
    program.programId
  )[0];

  console.log(`auth: ${auth.toBase58()}`);
  console.log(`transmuter: ${transmuter.toBase58()}`);

  const userMetaplex = Metaplex.make(anchor.getProvider().connection)
    .use(keypairIdentity(user))
    .use(mockStorage());

  const creatorMetaplex = Metaplex.make(anchor.getProvider().connection)
    .use(keypairIdentity(creator))
    .use(mockStorage());

  let inputCollection: any;
  let creatorCollection: any;
  let outputCollection: any;
  let inputMints = [];

  it("Airdrop", async () => {
    await Promise.all(
      [creator, user].map(async (key) => {
        return await anchor
          .getProvider()
          .connection.requestAirdrop(
            key.publicKey,
            100 * anchor.web3.LAMPORTS_PER_SOL
          );
      })
    ).then(confirmTxs);
  });

  it("Creates collections", async () => {
    inputCollection = await userMetaplex.nfts().create({
      name: "Input collection",
      symbol: "INPT",
      sellerFeeBasisPoints: 500,
      uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
      creators: [
        {
          address: creator.publicKey,
          share: 100,
        },
      ],
      isMutable: true,
    });

    outputCollection = await creatorMetaplex.nfts().create({
      name: "Output collection",
      symbol: "OUPT",
      sellerFeeBasisPoints: 500,
      uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
      creators: [
        {
          address: creator.publicKey,
          share: 100,
        },
      ],
      isMutable: true,
    });

    creatorCollection = await creatorMetaplex.nfts().create({
      name: "Creator collection",
      symbol: "INPT",
      sellerFeeBasisPoints: 500,
      uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
      creators: [
        {
          address: creator.publicKey,
          share: 100,
        },
      ],
      isMutable: true,
    });
  });

  it("mints input NFT", async () => {
    for (let i = 0; i < 2; i++) {
      let mint = await userMetaplex.nfts().create({
        name: `Generug input #${i + 1}`,
        symbol: "GNRG",
        sellerFeeBasisPoints: 500,
        uri: `https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI?Head=none&Background=blue&Outfit=cope&Breed=Shiba&Index=${
          i + 1
        }`,
        creators: [
          {
            address: creator.publicKey,
            share: 100,
          },
        ],
        collection: inputCollection.nft.address,
        isMutable: true,
      });
      inputMints.push(mint);

      await userMetaplex.nfts().verifyCollection({
        mintAddress: mint.nft.address,
        collectionMintAddress: inputCollection.nft.address,
      });

      console.log(`The nft #${i + 1}: ${mint.nft.address}`);
    }
  });

  it("mints creator input NFT", async () => {
    for (let i = 0; i < 1; i++) {
      let mint = await creatorMetaplex.nfts().create({
        name: `Generug creator #${i + 1}`,
        symbol: "GNRG",
        sellerFeeBasisPoints: 500,
        uri: `https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI?Head=none&Background=blue&Outfit=cope&Breed=Shiba&Index=${
          i + 1
        }`,
        creators: [
          {
            address: creator.publicKey,
            share: 100,
          },
        ],
        collection: creatorCollection.nft.address,
        isMutable: true,
      });

      await creatorMetaplex.nfts().verifyCollection({
        mintAddress: mint.nft.address,
        collectionMintAddress: creatorCollection.nft.address,
      });

      console.log(`The creator nft #${i + 1}: ${mint.nft.address}`);
    }
  });

  // it("close the transmuter", async () => {
  //   const tx = await program.methods
  //     .close()
  //     .accounts({
  //       creator: creator.publicKey,
  //     })
  //     .signers([creator])
  //     .rpc({
  //       skipPreflight: true,
  //     });
  // });

  it("creates the transmuter", async () => {
    const remainingAccounts = [];
    const remainingAccountsNftIndexer = [];

    const creatorNfts = await userMetaplex
      .nfts()
      .findAllByOwner({ owner: creator.publicKey });

    if (creatorNfts && creatorNfts.length > 0) {
      const nftsWithCollection = creatorNfts.filter(
        (nft) => nft.collection
      ) as {
        address: PublicKey;
        mintAddress: PublicKey;
        collection: { address: PublicKey };
      }[];

      const foundNft =
        nftsWithCollection.length > 0 &&
        nftsWithCollection[0].collection.address.toBase58() ===
          inputCollection.nft.address.toBase58();

      console.log(`found nft: ${foundNft}`);
      if (foundNft) {
        let indexes: { [key: string]: number } = {
          mint: 0,
          metadata: 0,
          ata: 0,
          creator_ata: 0,
        };

        //get mint
        indexes.mint = remainingAccounts.length;
        remainingAccounts.push({
          isSigner: false,
          isWritable: true,
          pubkey: nftsWithCollection[0].mintAddress,
        });

        //get metadata
        const metadata = await getMetadata(nftsWithCollection[0].mintAddress);
        indexes.metadata = remainingAccounts.length;
        remainingAccounts.push({
          isSigner: false,
          isWritable: true,
          pubkey: metadata,
        });

        //get ata
        const ata = await getOrCreateAssociatedTokenAccount(
          anchor.getProvider().connection,
          user,
          nftsWithCollection[0].mintAddress,
          user.publicKey,
          false
        );

        indexes.ata = remainingAccounts.length;
        remainingAccounts.push({
          isSigner: false,
          isWritable: true,
          pubkey: ata.address,
        });
        remainingAccountsNftIndexer.push(indexes);
      }
    }

    const owner = await getProgramAuthority(
      anchor.getProvider().connection,
      programId
    );
    console.log("owner: ", owner.toBase58());

    const wba = new PublicKey("3LSY4UTEFt7V7eGsiaAUDzn3iKAJFBPkYseXpdECFknF");
    console.log("wba: ", wba.toBase58());

    // const metadataCid = await storageClient.storeBlob(
    //   new File([JSON.stringify(traits)], "traits.json")
    // );
    // const traitsUri = `https://${metadataCid}.ipfs.nftstorage.link`;
    const traitsUri =
      "https://bafkreiaum2ncnoacx6la6o4anebrvgqgsoqymk62md5vw5mbbt2jvhfzfe.ipfs.nftstorage.link";

    const tx = await program.methods
      .create(
        seed,
        JSON.stringify([]),
        JSON.stringify([
          // {
          //   token_standard: "nft",
          //   collection: inputCollection.nft.address.toBase58(),
          //   method: "burn",
          //   amount: 1,
          //   rule: {
          //     name: "traits",
          //     rule_type: "match",
          //     trait_types: [
          //       ["Background", "blue"],
          //       ["Outfit", "cope"],
          //     ],
          //   },
          // },
          {
            token_standard: "nft",
            collection: inputCollection.nft.address.toBase58(),
            method: "burn",
            amount: 1,
          },
        ]),
        JSON.stringify([
          // {
          //   token_standard: "nft",
          //   collection: outputCollection.nft.address.toBase58(),
          //   method: "mint",
          //   amount: 1,
          //   mint: {
          //     title: "Generug output",
          //     symbol: "GNR",
          //     uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
          //   },
          // },
          {
            token_standard: "nft",
            collection: outputCollection.nft.address.toBase58(),
            method: "mint",
            amount: 1,
            // rule: {
            //   name: "split",
            //   rule_type: "mint",
            //   trait_types: [["Background", "*"]],
            // },
            mint: {
              title: "Generug split output",
              symbol: "SPLIT",
              uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
            },
          },
          // {
          //   token_standard: "nft",
          //   collection: outputCollection.nft.address.toBase58(),
          //   method: "mint",
          //   amount: 1,
          //   rule: {
          //     name: "merge",
          //     rule_type: "mint",
          //     trait_types: [
          //       ["Background", "*"],
          //       ["Outfit", "*"],
          //     ],
          //   },
          //   mint: {
          //     title: "Generug output",
          //     symbol: "GNR",
          //     uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
          //   },
          // },
        ]),
        traitsUri
      )
      .accounts({
        creator: creator.publicKey,
        auth,
        transmuter,
        systemProgram: SystemProgram.programId,
        owner,
        wba,
      })
      .remainingAccounts(remainingAccounts)
      .signers([creator])
      .rpc({
        skipPreflight: true,
      });
    console.log("Your transaction signature", tx);
  });

  const vaultSeed = new BN(randomBytes(8));

  const vaultAuth = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vaultAuth"),
      transmuter.toBytes(),
      user.publicKey.toBytes(),
      vaultSeed.toBuffer().reverse(),
    ],
    program.programId
  )[0];
  console.log(`vaultAuth: ${vaultAuth.toBase58()}`);

  it("handles input", async () => {
    const ata = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      inputMints[0].nft.address,
      user.publicKey,
      true
    );

    const vault = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      inputMints[0].nft.address,
      vaultAuth,
      true
    );

    const metadata = await getMetadata(inputMints[0].nft.address);

    const tx = await program.methods
      .sendInput(seed, vaultSeed)
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        mint: inputMints[0].nft.address,
        ata: ata.address,
        metadata: metadata,
        vaultAuth: vaultAuth,
        vault: vault.address,
        tokenProgram,
        transmuter,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc({
        skipPreflight: true,
      });

    console.log("DONE");
    console.log(tx);
  });

  it("should fail on same input", async () => {
    try {
      const ata = await getOrCreateAssociatedTokenAccount(
        anchor.getProvider().connection,
        user,
        inputMints[0].nft.address,
        user.publicKey,
        true
      );

      const vault = await getOrCreateAssociatedTokenAccount(
        anchor.getProvider().connection,
        user,
        inputMints[0].nft.address,
        vaultAuth,
        true
      );

      const metadata = await getMetadata(inputMints[0].nft.address);

      await program.methods
        .sendInput(seed, vaultSeed)
        .accounts({
          creator: creator.publicKey,
          user: user.publicKey,
          mint: inputMints[0].nft.address,
          ata: ata.address,
          metadata: metadata,
          vaultAuth: vaultAuth,
          vault: vault.address,
          tokenProgram,
          transmuter,
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

  // it("handles another input", async () => {
  //   const ata = await getOrCreateAssociatedTokenAccount(
  //     anchor.getProvider().connection,
  //     user,
  //     inputMints[1].nft.address,
  //     user.publicKey,
  //     true
  //   );

  //   const vault = await getOrCreateAssociatedTokenAccount(
  //     anchor.getProvider().connection,
  //     user,
  //     inputMints[1].nft.address,
  //     vaultAuth,
  //     true
  //   );

  //   const metadata = await getMetadata(inputMints[1].nft.address);

  //   const tx = await program.methods
  //     .sendInput(seed)
  //     .accounts({
  //       creator: creator.publicKey,
  //       user: user.publicKey,
  //       mint: inputMints[1].nft.address,
  //       ata: ata.address,
  //       metadata: metadata,
  //       vaultAuth: vaultAuth,
  //       vault: vault.address,
  //       tokenProgram,
  //       transmuter,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([user])
  //     .rpc({
  //       skipPreflight: true,
  //     });

  //   console.log("DONE");
  //   console.log(tx);
  // });

  const modifyComputeUnits =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 1000000,
    });

  const addPriorityFee = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 1,
  });

  it("should claim output", async () => {
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

    const tx = await program.methods
      .claimOutput(seed, vaultSeed)
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        vaultAuth,
        auth,
        transmuter,
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
      .preInstructions([modifyComputeUnits, addPriorityFee])
      .signers([user])
      .rpc({
        skipPreflight: true,
      });
    console.log(tx);
  });

  //How a creator can list his transmuters
  //How a creator can list completed vaultAuths
  //Find the inputMint from vaultAuth nfts

  //need a bookkeeping pda per creator to find all their transmuters
  //need a bookkeeping pda per transmute to find all the vaultAuth
  //find the nfts in the vaultAuth

  it("resolves an input", async () => {
    //creator ATA not used on burn
    //if all inputs are burn => use different method?
    const creatorAta = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      creator,
      inputMints[0].nft.address,
      creator.publicKey,
      true
    );

    const vault = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      user,
      inputMints[0].nft.address,
      vaultAuth,
      true
    );

    const metadata = await getMetadata(inputMints[0].nft.address);

    const tx = await program.methods
      .resolveInput(seed, vaultSeed)
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        mint: inputMints[0].nft.address,
        creatorAta: creatorAta.address,
        metadata: metadata,
        vaultAuth: vaultAuth,
        vault: vault.address,
        tokenProgram,
        transmuter,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator])
      .rpc({
        skipPreflight: true,
      });

    console.log("DONE");
    console.log(tx);
  });

  // it("Transmute", async () => {
  //   const transmuterInfo = await program.account.transmuter.fetch(transmuter);
  //   let transmuterInputs = JSON.parse(transmuterInfo.inputs) as InputInfo[];
  //   let transmuterOutputs = JSON.parse(transmuterInfo.outputs) as OutputInfo[];

  //   const handleOutputs = async (output: OutputInfo) => {
  //     for (let i = 0; i < output.amount; i++) {
  //       let mint = await createMint(
  //         anchor.getProvider().connection,
  //         user,
  //         auth,
  //         auth,
  //         0
  //       );
  //       let indexes: { [key: string]: number } = {
  //         mint: 0,
  //         metadata: 0,
  //         ata: 0,
  //         creator_ata: 0,
  //       };

  //       //Add mint
  //       console.log("Add mint");
  //       indexes.mint = remainingAccounts.length;
  //       remainingAccounts.push({
  //         isSigner: false,
  //         isWritable: true,
  //         pubkey: mint,
  //       });

  //       //Add metadata
  //       console.log("Add metadata");
  //       const metadata = await getMetadata(mint);
  //       indexes.metadata = remainingAccounts.length;
  //       remainingAccounts.push({
  //         isSigner: false,
  //         isWritable: true,
  //         pubkey: metadata,
  //       });

  //       //Add ata
  //       console.log("Add ata");
  //       const ata = await getOrCreateAssociatedTokenAccount(
  //         anchor.getProvider().connection,
  //         user,
  //         mint,
  //         user.publicKey,
  //         true
  //       );

  //       indexes.ata = remainingAccounts.length;
  //       remainingAccounts.push({
  //         isSigner: false,
  //         isWritable: true,
  //         pubkey: ata.address,
  //       });

  //       //Add owner ata
  //       console.log("Add owner ata");
  //       const creatorAta = await getOrCreateAssociatedTokenAccount(
  //         anchor.getProvider().connection,
  //         user,
  //         mint,
  //         creator.publicKey,
  //         true
  //       );
  //       indexes.creator_ata = remainingAccounts.length;
  //       remainingAccounts.push({
  //         isSigner: false,
  //         isWritable: true,
  //         pubkey: creatorAta.address,
  //       });

  //       //Add MasterEdition
  //       console.log("Add MasterEdition");
  //       const masterEdition = await getMasterEdition(mint);
  //       indexes.master_edition = remainingAccounts.length;
  //       remainingAccounts.push({
  //         isSigner: false,
  //         isWritable: true,
  //         pubkey: masterEdition,
  //       });

  //       remainingAccountsOutputIndexer.push(indexes);
  //     }
  //   };

  //   for (let [index, input] of Object.entries(transmuterInputs)) {
  //     switch (input.token_standard) {
  //       case "nft":
  //         let indexes: { [key: string]: number } = {
  //           mint: 0,
  //           metadata: 0,
  //           ata: 0,
  //           creator_ata: 0,
  //         };

  //         //Add mint
  //         indexes.mint = remainingAccounts.length;
  //         remainingAccounts.push({
  //           isSigner: false,
  //           isWritable: true,
  //           pubkey: inputMints[index].nft.address,
  //         });
  //         //Add metadata
  //         const metadata = await getMetadata(inputMints[index].nft.address);
  //         indexes.metadata = remainingAccounts.length;
  //         remainingAccounts.push({
  //           isSigner: false,
  //           isWritable: true,
  //           pubkey: metadata,
  //         });
  //         //Add ata
  //         const ata = await getOrCreateAssociatedTokenAccount(
  //           anchor.getProvider().connection,
  //           user,
  //           inputMints[index].nft.address,
  //           user.publicKey,
  //           true
  //         );
  //         indexes.ata = remainingAccounts.length;
  //         remainingAccounts.push({
  //           isSigner: false,
  //           isWritable: true,
  //           pubkey: ata.address,
  //         });
  //         if (input.method === "transfer") {
  //           //Add owner ata
  //           const creatorAta = await getOrCreateAssociatedTokenAccount(
  //             anchor.getProvider().connection,
  //             user,
  //             inputMints[index].nft.address,
  //             creator2.publicKey,
  //             true
  //           );
  //           indexes.creator_ata = remainingAccounts.length;
  //           remainingAccounts.push({
  //             isSigner: false,
  //             isWritable: true,
  //             pubkey: creatorAta.address,
  //           });
  //         }
  //         remainingAccountsInputIndexer.push(indexes);
  //         break;
  //       default:
  //         remainingAccountsInputIndexer.push({});
  //     }
  //   }

  //   for (let [_, output] of Object.entries(transmuterOutputs)) {
  //     switch (output.token_standard) {
  //       case "nft":
  //         if (output.rule?.name === "split") {
  //           for (let _ of Object.entries(transmuterInputs)) {
  //             await handleOutputs(output);
  //           }
  //         } else {
  //           await handleOutputs(output);
  //         }
  //         break;
  //       default:
  //         remainingAccountsOutputIndexer.push({});
  //     }
  //   }

  //   const modifyComputeUnits =
  //     anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
  //       units: 1000000,
  //     });

  //   const addPriorityFee = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice(
  //     {
  //       microLamports: 1,
  //     }
  //   );

  //   const tx = await program.methods
  //     .transmute(
  //       seed,
  //       JSON.stringify(remainingAccountsInputIndexer),
  //       JSON.stringify(remainingAccountsOutputIndexer)
  //     )
  //     .accounts({
  //       creator: creator.publicKey,
  //       user: user.publicKey,
  //       auth,
  //       transmuter,
  //       tokenProgram,
  //       associatedTokenProgram,
  //       tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
  //     })
  //     .preInstructions([modifyComputeUnits, addPriorityFee])
  //     .remainingAccounts(remainingAccounts)
  //     .signers([user])
  //     .rpc({
  //       skipPreflight: true,
  //     });
  //   console.log(tx);
  // });

  // it("Updates output uri", async () => {
  //   //should be done via server callback
  //   const transmuterInfo = await program.account.transmuter.fetch(transmuter);
  //   let transmuterOutputs = JSON.parse(transmuterInfo.outputs) as OutputInfo[];

  //   if (
  //     transmuterOutputs.some(
  //       (output) => output.rule.name === "merge" || output.rule.name === "split"
  //     )
  //   ) {
  //     const mints: anchor.web3.PublicKey[] = [];
  //     remainingAccountsOutputIndexer.forEach((outputIndex) =>
  //       mints.push(remainingAccounts[outputIndex.mint].pubkey)
  //     );
  //     for (let mint of mints) {
  //       const nft = await creatorMetaplex
  //         .nfts()
  //         .findByMint({ mintAddress: mint });
  //       const { data } = await axios.get(nft.uri);
  //       console.log(data);

  //       let queryString = new URLSearchParams(nft.uri.split("?")[1]);
  //       const attributes: { trait_type: string; value: string }[] = [];
  //       const layers: string[] = [];

  //       const transmuterTraits: TraitInfo[] = await axios
  //         .get(transmuterInfo.traitsUri)
  //         .then((res) => res.data);

  //       for (let [key, value] of queryString.entries()) {
  //         const foundTrait = transmuterTraits.find(
  //           (trait) => trait.trait_type === key && trait.value === value
  //         );
  //         if (foundTrait) {
  //           attributes.push({ trait_type: key, value: value });
  //           layers.push(foundTrait.image);
  //         }
  //       }

  //       const buffers: Buffer[] = [];
  //       for (let layer of layers) {
  //         const response = await axios.get(layer, {
  //           responseType: "arraybuffer",
  //         });
  //         const buffer = Buffer.from(response.data, "utf-8");
  //         buffers.push(buffer);
  //       }

  //       if (buffers[0]) {
  //         let outputBuffer = await sharp(buffers[0])
  //           .composite(
  //             buffers.map((buffer) => ({
  //               input: buffer,
  //               tile: true,
  //               blend: "over",
  //             }))
  //           )
  //           .toBuffer();
  //         const imageCid = await storageClient.storeBlob(
  //           new File([outputBuffer], "image.png")
  //         );
  //         const imageUri = `https://${imageCid}.ipfs.nftstorage.link`;
  //         const updatedMetadata = {
  //           ...data,
  //           attributes,
  //           image: imageUri,
  //           properties: {
  //             files: [
  //               {
  //                 uri: imageUri,
  //                 type: "image/png",
  //               },
  //             ],
  //           },
  //         };
  //         const metadataCid = await storageClient.storeBlob(
  //           new File([JSON.stringify(updatedMetadata)], "metadata.json")
  //         );
  //         let uri = `https://${metadataCid}.ipfs.nftstorage.link`;
  //         for (let [i, attribute] of Object.entries(attributes)) {
  //           const index = parseInt(i);
  //           if (index === 0) {
  //             uri += "?";
  //           }
  //           if (attribute) {
  //             uri += `${attribute.trait_type}=${attribute.value}`;
  //             if (index < attributes.length - 1) {
  //               uri += `&`;
  //             }
  //           }
  //         }

  //         await creatorMetaplex.nfts().update(
  //           {
  //             nftOrSft: nft,
  //             uri,
  //           },
  //           { commitment: "finalized" }
  //         );

  //         await creatorMetaplex
  //           .nfts()
  //           .verifyCreator(
  //             { mintAddress: mint, creator },
  //             { commitment: "finalized" }
  //           );

  //         await creatorMetaplex.nfts().verifyCollection({
  //           mintAddress: mint,
  //           collectionMintAddress: nft.collection.address,
  //         });
  //       }
  //     }
  //   }
  // });
});

// Helpers
const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor
    .getProvider()
    .connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    commitment
  );
};

const confirmTxs = async (signatures: string[]) => {
  await Promise.all(signatures.map(confirmTx));
};

const getMetadata = async (
  mint: anchor.web3.PublicKey
): Promise<anchor.web3.PublicKey> => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  )[0];
};

const getMasterEdition = async (
  mint: anchor.web3.PublicKey
): Promise<anchor.web3.PublicKey> => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    TOKEN_METADATA_PROGRAM_ID
  )[0];
};

const getProgramAuthority = async (
  c: Connection,
  programId: PublicKey
): Promise<PublicKey> => {
  const info = await c.getAccountInfo(programId, { commitment: "confirmed" });
  const dataAddress = new PublicKey(info.data.subarray(4));
  const dataAcc = await c.getAccountInfo(dataAddress);
  return new PublicKey(dataAcc.data.subarray(13, 45));
};

//solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s clones/metaplex.so
