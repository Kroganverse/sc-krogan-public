use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();

    blockchain.register_contract("file:output/game-faction.wasm", empty::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    elrond_wasm_debug::mandos_rs("mandos/init.scen.json", world());
}
