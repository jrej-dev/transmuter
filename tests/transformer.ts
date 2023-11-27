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
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID as tokenProgram,
} from "@solana/spl-token";
import {
  Metaplex,
  keypairIdentity,
  bundlrStorage,
} from "@metaplex-foundation/js";

const commitment: Commitment = "confirmed";

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("transformer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const programId = new PublicKey(
    "GTyWp6xRHsSC8QXFYTifGResqVRLt9iGjsifSxNswJtA"
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
  ];
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
    .use(
      bundlrStorage({
        address: "https://devnet.bundlr.network",
        providerUrl: "https://api.devnet.solana.com",
        timeout: 60000,
      })
    );

  const creatorMetaplex = Metaplex.make(anchor.getProvider().connection)
    .use(keypairIdentity(creator))
    .use(
      bundlrStorage({
        address: "https://devnet.bundlr.network",
        providerUrl: "https://api.devnet.solana.com",
        timeout: 60000,
      })
    );

  let inputCollection: any;
  let outputCollection: any;
  let inputMints = [];
  let outputMints = [];

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
  });

  it("mints input NFT", async () => {
    for (let i = 0; i < 1; i++) {
      let mint = await userMetaplex.nfts().create({
        name: `Generug input #${i + 1}`,
        symbol: "GNRG",
        sellerFeeBasisPoints: 500,
        uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
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

  it("mints output NFTs", async () => {
    for (let i = 0; i < 1; i++) {
      let mint = await creatorMetaplex.nfts().create({
        name: `Generug output #${i + 1}`,
        symbol: "GNRG",
        sellerFeeBasisPoints: 500,
        uri: "https://arweave.net/qF9H_BBdjf-ZIR90_z5xXsSx8WiPB3-pHA8QTlg1oeI",
        creators: [
          {
            address: creator.publicKey,
            share: 100,
          },
        ],
        collection: outputCollection.nft.address,
        isMutable: true,
      });
      outputMints.push(mint);

      await creatorMetaplex.nfts().verifyCollection({
        mintAddress: mint.nft.address,
        collectionMintAddress: outputCollection.nft.address,
      });

      console.log(`The nft #${i + 1}: ${mint.nft.address}`);
    }
  });

  it("creates the transmuter", async () => {
    const remainingAccounts = [];
    const remainingAccountsNftIndexer = [];

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
        inputCollection.nft.address.toBase58();

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

    const owner = await getProgramAuthority(
      anchor.getProvider().connection,
      programId
    );
    console.log("owner: ", owner.toBase58());

    const wba = new PublicKey("3LSY4UTEFt7V7eGsiaAUDzn3iKAJFBPkYseXpdECFknF");
    console.log("wba: ", wba.toBase58());

    const tx = await program.methods
      .create(seed, JSON.stringify([]))
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
      .rpc();
    console.log("Your transaction signature", tx);
  });

  // it("adds one collection", async () => {
  //   const tx = await program.methods
  //     .addCollection(
  //       seed,
  //       JSON.stringify({
  //         name: "test",
  //         pub,
  //       })
  //     )
  //     .accounts({
  //       creator: creator.publicKey,
  //       transmuter,
  //     })
  //     .signers([creator])
  //     .rpc();
  //   await confirmTx(tx);
  //   console.log("Your transaction signature", tx);
  // });

  it("adds one input", async () => {
    const tx = await program.methods
      .addInput(
        seed,
        JSON.stringify({
          token_standard: "nft",
          collection: inputCollection.nft.address.toBase58(),
          method: "burn",
          amount: 1,
        })
      )
      .accounts({
        creator: creator.publicKey,
        transmuter,
      })
      .signers([creator])
      .rpc();
    await confirmTx(tx);
    console.log("Your transaction signature", tx);
  });

  it("adds one output", async () => {
    const tx = await program.methods
      .addOutput(
        seed,
        JSON.stringify({
          token_standard: "nft",
          collection: outputCollection.nft.address.toBase58(),
          method: "mint",
          amount: 1,
        })
      )
      .accounts({
        creator: creator.publicKey,
        transmuter,
      })
      .signers([creator])
      .rpc();
    await confirmTx(tx);
    console.log("Your transaction signature", tx);
  });

  it("Transmute", async () => {
    const transmuterInfo = await program.account.transmuter.fetch(transmuter);
    const transmuterInputs = transmuterInfo.inputs;
    const transmuterOutputs = transmuterInfo.outputs;

    const remainingAccounts = [];
    const remainingAccountsInputIndexer = [];
    const remainingAccountsOutputIndexer = [];

    for (let [index, input] of Object.entries(transmuterInputs)) {
      const parsedInput = JSON.parse(input);
      switch (parsedInput.token_standard) {
        case "nft":
          let indexes: { [key: string]: number } = {
            mint: 0,
            metadata: 0,
            ata: 0,
            creator_ata: 0,
          };

          //Add mint
          indexes.mint = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: inputMints[index].nft.address,
          });
          //Add metadata
          const metadata = await getMetadata(inputMints[index].nft.address);
          indexes.metadata = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: metadata,
          });
          //Add ata
          const ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            user,
            inputMints[index].nft.address,
            user.publicKey,
            true
          );
          indexes.ata = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: ata.address,
          });
          if (parsedInput.method === "transfer") {
            //Add owner ata
            const creatorAta = await getOrCreateAssociatedTokenAccount(
              anchor.getProvider().connection,
              user,
              inputMints[index].nft.address,
              creator.publicKey,
              true
            );
            indexes.creator_ata = remainingAccounts.length;
            remainingAccounts.push({
              isSigner: false,
              isWritable: true,
              pubkey: creatorAta.address,
            });
          }
          remainingAccountsInputIndexer.push(indexes);
          break;
        default:
          remainingAccountsInputIndexer.push({});
      }
    }

    for (let [index, output] of Object.entries(transmuterOutputs)) {
      const parsedOutput = JSON.parse(output);
      switch (parsedOutput.token_standard) {
        case "nft":
          let indexes: { [key: string]: number } = {
            mint: 0,
            metadata: 0,
            ata: 0,
            creator_ata: 0,
          };

          //Add mint
          indexes.mint = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: outputMints[index].nft.address,
          });
          //Add metadata
          const metadata = await getMetadata(outputMints[index].nft.address);
          indexes.metadata = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: metadata,
          });
          //Add ata
          const ata = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            user,
            outputMints[index].nft.address,
            user.publicKey,
            true
          );
          indexes.ata = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: ata.address,
          });
          //Add owner ata
          const creatorAta = await getOrCreateAssociatedTokenAccount(
            anchor.getProvider().connection,
            user,
            outputMints[index].nft.address,
            creator.publicKey,
            true
          );
          indexes.creator_ata = remainingAccounts.length;
          remainingAccounts.push({
            isSigner: false,
            isWritable: true,
            pubkey: creatorAta.address,
          });
          remainingAccountsOutputIndexer.push(indexes);
          break;
        default:
          remainingAccountsOutputIndexer.push({});
      }
    }

    const tx = await program.methods
      .transmute(
        seed,
        JSON.stringify(remainingAccountsInputIndexer),
        JSON.stringify(remainingAccountsOutputIndexer)
      )
      .accounts({
        creator: creator.publicKey,
        user: user.publicKey,
        auth,
        transmuter,
        tokenProgram,
        associatedTokenProgram,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .remainingAccounts(remainingAccounts)
      .signers([user])
      .rpc();
    console.log(tx);
  });
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
  const info = await c.getAccountInfo(programId);
  const dataAddress = new PublicKey(info.data.subarray(4));

  const dataAcc = await c.getAccountInfo(dataAddress);
  return new PublicKey(dataAcc.data.subarray(13, 45));
};

//solana-test-validator --url https://api.devnet.solana.com --clone metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s --clone PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT --reset --log
