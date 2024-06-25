// import "./case1";
import * as anchor from "@coral-xyz/anchor";
import { Transformer, IDL } from "../target/types/transformer";

anchor.setProvider(anchor.AnchorProvider.env());

export const programId = new anchor.web3.PublicKey(
  "H8SJKV7T4egtcwoA2HqSCNYeqrTJuA7SDSeZNrAgMmpf"
);

export const program = new anchor.Program<Transformer>(
  IDL,
  programId,
  anchor.getProvider()
);

import "./case0";
import "./case1";
import "./case2";
import './case3';

//solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s clones/metaplex.so