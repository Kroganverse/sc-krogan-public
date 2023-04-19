// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           17
// Async Callback (empty):               1
// Total number of exported functions:  19

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    game_faction
    (
        registerBalancer
        registerProposalType
        registerUser
        swapUser
        removeUser
        propose
        vote
        concludeProposal
        removeProposal
        getBalancer
        getStakingPool
        getUserCount
        isRegistered
        getDaoIndex
        getDaoItem
        getUserVoted
        getProposalTypeCost
    )
}

elrond_wasm_node::wasm_empty_callback! {}
