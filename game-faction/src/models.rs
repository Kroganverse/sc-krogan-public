elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(
	TopEncode, TopDecode, Clone, PartialEq, Eq, TypeAbi
)]
pub enum VoteType {
    AbstainVote,
    UpVote,
    DownVote,
}

#[derive(
	TopEncode, TopDecode, Clone, PartialEq, Eq, 
	NestedDecode, NestedEncode, TypeAbi,
)]
pub enum ProposalStatus {
    Pending,
    Failed,
    Succeeded,
}

#[derive(TopEncode, TopDecode, TypeAbi, Clone)]
pub struct DaoItem<M: ManagedTypeApi> {
	pub id: u32,
	pub owner: ManagedAddress<M>,
	pub deadline: u64,
	pub proposal: ManagedBuffer<M>, // tweet size text
    pub proposal_type: ManagedBuffer<M>, // eg: starbase
    pub args: ManagedBuffer<M>, // used by BE, in the form: type:value;
	pub up_votes: BigUint<M>,
	pub down_votes: BigUint<M>,
	pub abstain_votes: BigUint<M>,
	pub votes: u32,
	pub status: ProposalStatus,
}