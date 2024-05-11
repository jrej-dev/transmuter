import * as anchor from "@coral-xyz/anchor";
import { Metadata, Nft, PublicKey, Sft } from "@metaplex-foundation/js";
import { InputInfo } from "./types";
import { Transformer } from "../target/types/transformer";

const isMatching = async (
  nft: Metadata | Nft | Sft,
  inputInfo: InputInfo
): Promise<boolean> => {
  let isMatch = false;
  switch (inputInfo.token_standard) {
    case "nft":
    default: {
      isMatch = nft.collection?.address.toBase58() === inputInfo.collection;

      if (isMatch) {
        if (inputInfo.rule) {
          isMatch = false;

          switch (inputInfo.rule.name) {
            case "traits":
              let parsedUrl = new URLSearchParams(nft.uri);
              console.log(parsedUrl);

              break;
            default:
              console.log("rule not found");
          }
        }
      }
    }
  }
  return isMatch;
};

export const getInputMatch = async (
  program: anchor.Program<Transformer>,
  transmuter: PublicKey,
  vaultAuth: PublicKey,
  vaultAuthNft: Metadata | Nft | Sft
): Promise<InputInfo | undefined> => {
  const transmuterInfo = await program.account.transmuter.fetch(transmuter);
  const vaultInfo = await program.account.vaultAuth.fetch(vaultAuth);
  let inputInfos = JSON.parse(transmuterInfo.inputs) as InputInfo[];

  for (let [index, inputInfo] of Object.entries(inputInfos)) {
    if (!Array.from(vaultInfo.handledInputIndexes).includes(parseInt(index))) {
      continue;
    }
    const isMatch = await isMatching(vaultAuthNft, inputInfo);
    if (isMatch) {
      return inputInfo;
    }
  }
};
