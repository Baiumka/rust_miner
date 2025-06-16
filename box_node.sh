dfx ledger fabricate-cycles --canister miner_backend --all
cargo build --release --target wasm32-unknown-unknown
dfx deploy miner_backend
dfx canister call miner_backend register Console
dfx canister call icp_ledger_canister icrc2_approve '(
  record {
    spender = record { owner = principal "bd3sg-teaaa-aaaaa-qaaba-cai"; subaccount = null };
    amount = 500_620_000: nat;
    from_subaccount = null;
    expected_allowance = null;
    expires_at = null;
    fee = null;
    memo = null;
    created_at_time = null;
  }
)'
dfx canister call miner_backend get_my_allowance
dfx canister call miner_backend create_box 500_000_000
dfx canister call miner_backend get_all_boxes
dfx ledger fabricate-cycles --canister miner_backend --all
dfx canister call miner_backend create_miner

