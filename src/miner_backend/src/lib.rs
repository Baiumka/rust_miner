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

const BOX_CANISTER_CYCLES: u128 = 10_000_000_000_000;
const MIN_BOX_COST: u64 = 500_000_000;
const FEE: u64 = 10_000;

thread_local! {
    static USERS: std::cell::RefCell<BTreeMap<String, T::User>> = std::cell::RefCell::new(BTreeMap::new());
    static BOXES: std::cell::RefCell<BTreeMap<String, String>> = std::cell::RefCell::new(BTreeMap::new());//box principal, user principal
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

fn search_empty_canister() -> Option<String> {
    BOXES.with(|boxes| {
        boxes
            .borrow()
            .iter()
            .find(|(_, v)| v == &"NONE")
            .map(|(k, _)| k.clone())
    })
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
fn get_all_boxes() -> Vec<T::BoxInfo> {    
    let mut result = vec![];

    // Получаем BOXES
    BOXES.with(|boxes_ref| {
        USERS.with(|users_ref| {
            let boxes = boxes_ref.borrow();
            let users = users_ref.borrow();

            for (canister_id, user_id) in boxes.iter() {
                if let Some(user) = users.get(user_id) {
                    result.push(T::BoxInfo {
                        canister_id: canister_id.clone(),
                        user: user.clone(),
                    });
                }
            }
        });
    });
    result
}

#[ic_cdk::update]
async fn create_box(award: Nat) -> Result<String, String> {    
    let min_cost  = Nat::from(MIN_BOX_COST);
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
                let new_canister_id: String; 
                let empty_canister = search_empty_canister();
                if let Some(key) = empty_canister {
                    new_canister_id = key;
                } else {
                    new_canister_id = create_box_canister().await;
                }
                
                let maube_new_principal = candid::Principal::from_text(new_canister_id.clone());
                match maube_new_principal {
                    Ok(new_principal) => {
                        match transfer_from(award, ic_cdk::caller(), new_principal).await {
                            Ok(index) => {                                            
                                BOXES.with(|boxes: &std::cell::RefCell<BTreeMap<String, String>>| {
                                    boxes.borrow_mut().insert(new_canister_id.clone(), ic_cdk::caller().to_text());
                                });
                                let result_info = call_get_principal(new_canister_id).await;
                                Ok(result_info.unwrap())
                            },                            
                            Err(e) => 
                            {
                                BOXES.with(|boxes: &std::cell::RefCell<BTreeMap<String, String>>| {
                                    boxes.borrow_mut().insert(new_canister_id.clone(), "NONE".to_string());
                                });
                                Err(e)
                            }
                        }     
                    }
                    Err(e) => {Err("Create canister error".to_string()) }
                }
                           
            }
            else {
               Err("You have no enough ICP".to_string()) 
            }
        },
        Err(e) => Err(e),
    }
}

async fn transfer_from(amount: Nat, from: Principal, to: Principal) -> Result<Nat, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {

            let transfer_from_args = T::ICRC2TransferFromArgs {
                from: T::ICRCAccount { owner: from, subaccount: None },
                memo: None,
                amount: amount,
                spender_subaccount: None,
                fee: None,
                to: T::ICRCAccount { owner: to, subaccount: None },
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

