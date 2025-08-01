use candid::{CandidType};
use serde::{Deserialize, Serialize};
use candid::{Nat, Principal};

pub const LEDGER_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[derive(CandidType, Deserialize, Clone)]
pub struct User {
    pub nickname: String,    
}


#[derive(CandidType, Deserialize, Clone)]
pub struct BoxInfo {
    pub user: String, 
    pub canister_id: String,    
    pub reg_date: u64,
    pub end_date: u64,
    pub is_end: bool
}


#[derive(CandidType, Deserialize, Clone)]
pub struct BoxWithCount {
    pub username: String,
    pub miner_count: u32,
    pub end_date: u64,
    pub reg_date: u64,
    pub canister_id: String,
    pub user_miners: Vec<Miner>
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Miner {
    pub user: String, 
    pub box_id: String,
    pub canister_id: String,    
    pub reg_date: u64,
    pub end_date: u64,
    pub is_end: bool
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TransferArg {
    pub from_subaccount: Option<Vec<u8>>,
    pub to: ICRCAccount,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}


#[derive(candid::CandidType)]
pub struct AccountBalanceArgs {
    pub account: Vec<u8>
}

#[derive(candid::CandidType, Deserialize)]
pub struct BalanceArgs {
    pub e8s: u64
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ICRCAccount {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ICRC2ApproveArgs {
    pub spender: ICRCAccount,
    pub amount: Nat,
    pub from_subaccount: Option<Vec<u8>>,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum TranferResult {
    Ok(Nat), 
    Err(TransferError),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: u64, message: String },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ICRC2ApproveResult {
    Ok(Nat),
    Err(ICRC2ApproveError),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ICRC2ApproveError {
    BadFee { expected_fee: Nat },
    InsufficientFunds { balance: Nat },
    AllowanceChanged { current_allowance: Nat },
    Expired { ledger_time: u64 },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ICRC2TransferFromArgs {
    pub spender_subaccount: Option<Vec<u8>>,
    pub from: ICRCAccount,
    pub to: ICRCAccount,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ICRC2TransferFromResult {
    Ok(Nat), // Block index
    Err(ICRC2TransferFromError),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub enum ICRC2TransferFromError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    InsufficientAllowance { allowance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ICRC2AllowanceArgs {
    pub account: ICRCAccount,
    pub spender: ICRCAccount,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ICRC2Allowance {
    pub allowance: Nat,
    pub expires_at: Option<u64>,
}