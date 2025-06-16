#[test_only]
module nft::nft_tests;
// uncomment this line to import the module
use kiosk::kiosk;
use sui::tx_context::{TxContext, new_tx_context};

const ENotImplemented: u64 = 0;

#[test]
fun test_nft() {
    // pass
}

#[test, expected_failure(abort_code = ::nft::nft_tests::ENotImplemented)]
fun test_nft_fail() {
    abort ENotImplemented
}
