export type InputInfo = {
  amount: number;
  collection: String;
  method: String;
  token_standard: String;
  rule?: Rule;
};

export type OutputInfo = {
  amount: number;
  collection: String;
  method: String;
  token_standard: String;
  rule?: Rule;
  uri?: String;
  mint?: MintInfo;
};

export type TraitInfo = {
  name: string;
  description: string;
  trait_type: string;
  value: string;
  uri: string;
  image: string;
};

type Rule = {
  name: string;
  rule_type: string;
  trait_types: [[string, string]];
};

type MintInfo = {
  title: string;
  symbol: string;
  uri: string;
};
