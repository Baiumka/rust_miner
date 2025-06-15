use ic_cdk::api::call::call;
use ic_cdk::api;
use candid::{encode_args, decode_args};
use ic_cdk::api::stable::{stable_write, stable_read};
use ic_cdk::api::management_canister::main::{
    CreateCanisterArgument, create_canister, InstallCodeArgument, install_code, CanisterInstallMode,
};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use candid::{Nat, Principal};
use std::collections::BTreeMap;
use types as T;

const BOX_NODE_WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/box_node.wasm");
const MINER_NODE_WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/miner_node.wasm");

const BOX_CANISTER_CYCLES: u128 = 10_000_000_000_000;
const MIN_BOX_COST: u64 = 500_000_000;
const MIN_MINER_COST: u64 = 500_000;
const FEE: u64 = 10_000;
const LOTTERY_TIME: u64 = 1 * 1 * 1 * 60; //days hours mins seconds
const MINER_TIME: u64 = 1 * 1 * 1 * 60; //days hours mins seconds

thread_local! {
    static USERS: std::cell::RefCell<BTreeMap<String, T::User>> = std::cell::RefCell::new(BTreeMap::new());
    static BOXES: std::cell::RefCell<BTreeMap<String, T::BoxInfo>> = std::cell::RefCell::new(BTreeMap::new());
    static MINERS: std::cell::RefCell<BTreeMap<String, T::Miner>> = std::cell::RefCell::new(BTreeMap::new());
    static BOX_MINER: std::cell::RefCell<BTreeMap<String, String>> = std::cell::RefCell::new(BTreeMap::new());
    static SUB_INDEX: std::cell::RefCell<u32> = std::cell::RefCell::new(0);
}


fn get_user_by_princ(principal: String) -> Option<T::User> {
    USERS.with(|users| users.borrow().get(&principal).cloned())
}

#[ic_cdk::query]
fn get_user() -> Result<T::User, String>{
    let principal_text = ic_cdk::caller().to_text();
    let maybe_user = get_user_by_princ(principal_text.clone());
    if let Some(user) = maybe_user {
        Ok(user)
    } else {
        return Err("User not exist".to_string())
    }
}

#[ic_cdk::query]
fn show_all_users() -> Vec<(String, T::User)> {
    USERS.with(|users| users.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

#[ic_cdk::update]
fn register(nickname: String) -> Result<T::User, String>  {
    let principal_text = ic_cdk::caller().to_text();
    let maybe_user = get_user_by_princ(principal_text.clone());
    if maybe_user.is_some() {
        return Err("User already exist".to_string())
    }

    let user = T::User { nickname }.clone();    
    USERS.with(|users: &std::cell::RefCell<BTreeMap<String, T::User>>| {
        users.borrow_mut().insert(principal_text, user.clone());
    });
    Ok(user)
}

async fn create_box_canister() -> String {
    let create_args: CreateCanisterArgument = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
        })
    };
        
    let canister_record = create_canister(create_args, BOX_CANISTER_CYCLES).await.unwrap();
    let canister_id = canister_record.0.canister_id;
    let install_args = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: BOX_NODE_WASM.to_vec(),
        arg: vec![],
    };
    install_code(install_args).await.unwrap(); 
    return canister_id.to_text();
}

async fn create_miner_canister() -> String {
    let create_args: CreateCanisterArgument = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![ic_cdk::id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
        })
    };
        
    let canister_record = create_canister(create_args, BOX_CANISTER_CYCLES).await.unwrap();
    let canister_id = canister_record.0.canister_id;
    let install_args = InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: MINER_NODE_WASM.to_vec(),
        arg: vec![],
    };
    install_code(install_args).await.unwrap(); 
    return canister_id.to_text();
}

#[ic_cdk::update]
async fn call_get_principal(target_canister: String) -> Result<String, String> {
    let maybe_principal = candid::Principal::from_text(target_canister);
    match maybe_principal {
        Ok(principal) => {
            match call::<(), (String,)>(principal, "get_principal", ()).await {
                Ok((principal_str,)) => Ok(principal_str),
                Err(e) => Err(format!("Call failed: {:?}", e)),
            }
        }
        Err(err) => {
            Err("User already exist".to_string())
        }
    }
}



async fn get_balance(target_principal: Principal) -> Result<u64, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {
            let args = T::ICRCAccount {
                owner: target_principal, 
                subaccount: None,
            };
            match call::<(T::ICRCAccount,), (Vec<u8>,)>(ledger_principal, "account_identifier", (args,)).await {
                Ok((account_nat8,)) => {
                    let balance_args = T::AccountBalanceArgs {                        
                        account: account_nat8
                    };
                    match call::<(T::AccountBalanceArgs,), (T::BalanceArgs,)>(ledger_principal, "account_balance", (balance_args,)).await {
                        Ok((balance,)) => {
                            Ok(balance.e8s)
                        },
                        Err(e) => Err(format!("account_balance Call failed: {:?}", e)),
                    }
                },
                Err(e) => Err(format!("account_identifier Call failed: {:?}", e)),
            }
        }
        Err(err) => {
            Err("Failed get balance".to_string())
        }
    }
}


async fn get_allowance(target_principal: Principal) -> Result<Nat, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {
            let args = T::ICRC2AllowanceArgs {
                account: T::ICRCAccount { owner: target_principal, subaccount: None },
                spender: T::ICRCAccount { owner: api::id(), subaccount: None },
            };
                        
            match call::<(T::ICRC2AllowanceArgs,), (T::ICRC2Allowance,)>(ledger_principal, "icrc2_allowance", (args,)).await {
                Ok((balance,)) => {
                    Ok(balance.allowance)
                },
                Err(e) => Err(format!("icrc2_allowance Call failed: {:?}", e)),
            }            
        }
        Err(err) => {
            Err("Failed get balance".to_string())
        }
    }
}



#[ic_cdk::update]
async fn get_my_balance() -> Result<u64, String> {    
    match get_balance(ic_cdk::caller()).await {
        Ok(balance64) => {            
            Ok(balance64)
        },
        Err(e) => Err(e),
    }
}

#[ic_cdk::update]
async fn get_my_allowance() -> Result<Nat, String> {    
    match get_allowance(ic_cdk::caller()).await {
        Ok(balance) => {            
            Ok(balance)
        },
        Err(e) => Err(e),
    }
}

#[ic_cdk::query]
fn get_all_boxes() -> Vec<T::BoxWithCount> {
    let mut result = vec![];

    BOXES.with(|boxes_ref| {
        BOX_MINER.with(|box_miner_ref| {
            let boxes = boxes_ref.borrow();
            let box_miner = box_miner_ref.borrow();

            for (box_id, box_info) in boxes.iter() {
                // Count how many miners reference this box_id
                let count = box_miner.values().filter(|v| *v == box_id).count() as u32;

                result.push(T::BoxWithCount {
                    boxx: box_info.clone(),
                    miner_count: count,
                });
            }
        });
    });

    result
}

#[ic_cdk::update]
async fn create_miner(box_id: String, award: Nat) -> Result<String, String> {    
    if(award < MIN_MINER_COST)
    {
        return Err(format!("Minimum cost: {:?} ICP", MIN_MINER_COST))
    }
    let maybe_user = get_user_by_princ(ic_cdk::caller().to_text());
    if maybe_user.is_none() {
        return Err("User not found".to_string())
    }
    match get_allowance(ic_cdk::caller()).await {
        Ok(balance) => { 
            if(balance >= award.clone() + FEE)
            {  
                let index = SUB_INDEX.with(|i| {
                    let mut i = i.borrow_mut();
                    *i += 1;
                    *i
                });                        
                let sub = create_subaccount(api::id(), index);
                let result_sub: Option<Vec<u8>> = Some(sub.to_vec());       
                match transfer_from(award, ic_cdk::caller(), result_sub.clone()).await {
                    Ok(index) => {                              
                        let new_canister_id = create_miner_canister().await;
                        match call::<(Vec<u8>,), ()>(candid::Principal::from_text(new_canister_id.clone()).unwrap(), "init", (result_sub.clone().unwrap(),)).await {
                            Ok(()) => {},
                            Err(e) => return Err(format!("Init Call failed: {:?}", e)),
                        }  
                        let now = api::time();                                                                
                        let new_miner_info = T::Miner {
                            user: ic_cdk::caller().to_text(),
                            canister_id: new_canister_id.clone(),
                            box_id: box_id.clone(),
                            reg_date: now,
                            end_date: now + MINER_TIME,
                        };    
                        MINERS.with(|miners: &std::cell::RefCell<BTreeMap<String, T::Miner>>| {
                            miners.borrow_mut().insert(new_canister_id.clone(), new_miner_info);
                        });
                        BOX_MINER.with(|map| {
                            map.borrow_mut().insert(new_canister_id.clone(), box_id.clone());
                        });
                        Ok(new_canister_id)
                    },                            
                    Err(e) => 
                    {                        
                        Err(e)
                    }                 
                }
            }
            else {
                Err("You have no enough ICP".to_string())
            }
        }
        Err(e) => Err(e)
    }   
}

#[ic_cdk::update]
async fn create_box(award: Nat) -> Result<String, String> {        
    if(award < MIN_BOX_COST)
    {
        return Err(format!("Minimum cost: {:?} ICP", MIN_BOX_COST))
    }
    let maybe_user = get_user_by_princ(ic_cdk::caller().to_text());
    if maybe_user.is_none() {
        return Err("User not found".to_string())
    }
    match get_allowance(ic_cdk::caller()).await {
        Ok(balance) => {            
            if(balance >= award.clone() + FEE)
            {                                                                                    
                let index = SUB_INDEX.with(|i| {
                    let mut i = i.borrow_mut();
                    *i += 1;
                    *i
                });                        
                let sub = create_subaccount(api::id(), index);
                let result_sub: Option<Vec<u8>> = Some(sub.to_vec());                                
                match transfer_from(award, ic_cdk::caller(), result_sub.clone()).await {
                    Ok(index) => {                              
                        let new_canister_id = create_box_canister().await;
                        match call::<(Vec<u8>,), ()>(candid::Principal::from_text(new_canister_id.clone()).unwrap(), "init", (result_sub.clone().unwrap(),)).await {
                            Ok(()) => {},
                            Err(e) => return Err(format!("Init Call failed: {:?}", e)),
                        }  
                        let now = api::time();                                                                
                        let new_box_info = T::BoxInfo {
                            user: ic_cdk::caller().to_text(),
                            canister_id: new_canister_id.clone(),
                            reg_date: now,
                            end_date: now + LOTTERY_TIME,
                        };    
                        BOXES.with(|boxes: &std::cell::RefCell<BTreeMap<String, T::BoxInfo>>| {
                            boxes.borrow_mut().insert(new_canister_id.clone(), new_box_info);
                        });
                        let result_info = call_get_principal(new_canister_id).await;
                        Ok(result_info.unwrap())
                    },                            
                    Err(e) => 
                    {                        
                        Err(e)
                    }
                }                                                                    
            }
            else {
               Err("You have no enough ICP".to_string()) 
            }
        },
        Err(e) => Err(e),
    }
}

async fn transfer_from(amount: Nat, from: Principal, to_sub: Option<Vec<u8>>) -> Result<Nat, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {

            let transfer_from_args = T::ICRC2TransferFromArgs {
                from: T::ICRCAccount { owner: from, subaccount: None },
                memo: None,
                amount: amount,
                spender_subaccount: None,
                fee: None,
                to: T::ICRCAccount { owner: api::id(), subaccount: to_sub },
                created_at_time: None,
            };

            match call::<(T::ICRC2TransferFromArgs,), (T::ICRC2TransferFromResult,)>(ledger_principal, "icrc2_transfer_from", (transfer_from_args,)).await {
                Ok((approve_result,)) => {
                    match approve_result {
                        T::ICRC2TransferFromResult::Ok(index) => {
                            Ok(index)
                        },
                        T::ICRC2TransferFromResult::Err(e) => {
                            let json_str = serde_json::to_string(&e);
                            match json_str {
                                Ok(str) => {
                                    Err("tranfer error: ".to_string() + &str)
                                },
                                Err(e) => {
                                    Err("Unkown tranfer error".to_string())
                                },
                            }                            
                        }
                    }
                },
                Err(e) => Err(format!("icrc2_transfer_from Call failed: {:?}", e)),
            }
        }
        Err(err) => {
            Err("Bad ledger canister".to_string())
        }
    }
}

pub fn create_subaccount(user_id: Principal, index: u32) -> [u8; 32] {
    let mut subaccount: [u8; 32] = [0; 32];

    subaccount[0] = u8::try_from(index).unwrap();

    let user_bytes = user_id.as_slice(); 

    for (i, byte) in user_bytes.iter().enumerate() {
        if i + 1 < 32 {
            subaccount[i + 1] = *byte;
        }
    }

    subaccount
}