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
use types::{self as T, BoxWithCount};
use ic_cdk_timers::{set_timer};
use std::time::Duration;
use ic_cdk::api::print;
use ic_cdk::api::management_canister::main::raw_rand;

const BOX_NODE_WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/box_node.wasm");
const MINER_NODE_WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/miner_node.wasm");

const CANISTER_CYCLES: u128 = 100_000_000_000;
const MIN_BOX_COST: u64 = 500_000_000;
const MIN_MINER_COST: u64 = 500_000;
const FEE: u64 = 10_000;
const LOTTERY_TIME: u64 = 1 * 1 * 1 * 60; //days hours mins seconds
const MINER_TIME: u64 = 1 * 1 * 1 * 20; //days hours mins seconds

thread_local! {
    static USERS: std::cell::RefCell<BTreeMap<String, T::User>> = std::cell::RefCell::new(BTreeMap::new());
    static BOXES: std::cell::RefCell<BTreeMap<String, T::BoxInfo>> = std::cell::RefCell::new(BTreeMap::new());
    static MINERS: std::cell::RefCell<BTreeMap<String, T::Miner>> = std::cell::RefCell::new(BTreeMap::new());
    static BOX_MINER: std::cell::RefCell<BTreeMap<String, String>> = std::cell::RefCell::new(BTreeMap::new());
    static SUB_INDEX: std::cell::RefCell<u32> = std::cell::RefCell::new(0);
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    USERS.with(|users| {
        BOXES.with(|boxes| {
            MINERS.with(|miners| {
                BOX_MINER.with(|box_miner| {
                    SUB_INDEX.with(|sub_index| {
                        ic_cdk::storage::stable_save((
                            users.borrow().clone(),
                            boxes.borrow().clone(),
                            miners.borrow().clone(),
                            box_miner.borrow().clone(),
                            *sub_index.borrow(),
                        )).expect("Failed to save state to stable memory");
                    });
                });
            });
        });
    });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    let (users, boxes, miners, box_miner, sub_index): (
        BTreeMap<String, T::User>,
        BTreeMap<String, T::BoxInfo>,
        BTreeMap<String, T::Miner>,
        BTreeMap<String, String>,
        u32,
    ) = ic_cdk::storage::stable_restore().expect("Failed to restore state from stable memory");

    USERS.with(|u| *u.borrow_mut() = users);
    BOXES.with(|b| *b.borrow_mut() = boxes);
    MINERS.with(|m| *m.borrow_mut() = miners);
    BOX_MINER.with(|bm| *bm.borrow_mut() = box_miner);
    SUB_INDEX.with(|si| *si.borrow_mut() = sub_index);
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
        
    let canister_record = create_canister(create_args, CANISTER_CYCLES).await.unwrap();
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
        
    let canister_record = create_canister(create_args, CANISTER_CYCLES).await.unwrap();
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



async fn get_balance(target_principal: Principal, sub: Option<Vec<u8>>) -> Result<u64, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {
            let args = T::ICRCAccount {
                owner: target_principal, 
                subaccount: sub,
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
    match get_balance(ic_cdk::caller(), None).await {
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
        let boxes = boxes_ref.borrow();

        for (box_id, box_info) in boxes.iter() {
            if box_info.is_end {
                continue; 
            }
            let active_miners = get_active_miners(box_id.clone());
            let maybe_username = get_user_by_princ(box_info.clone().user);
            let username: String = match maybe_username {
                Some(user) => user.nickname,
                None => "Unknown".to_string(),
            };
            if !active_miners.is_empty() {
                result.push(T::BoxWithCount {
                    username: username,
                    miner_count: active_miners.len() as u32,
                    end_date: box_info.clone().end_date,
                    reg_date: box_info.clone().reg_date,
                    canister_id: box_info.clone().canister_id,
                });
            }
            else {
                result.push(T::BoxWithCount {
                    username: username,
                    miner_count: 0,
                    end_date: box_info.clone().end_date,
                    reg_date: box_info.clone().reg_date,
                    canister_id: box_info.clone().canister_id,
                });
            }
        }
    });
    result.sort_by(|a, b| b.end_date.cmp(&a.end_date));
    result
}

fn get_active_miners(box_id: String) -> Vec<T::Miner> {
    MINERS.with(|miners_ref| {
        let miners = miners_ref.borrow();
        miners
            .values()
            .filter(|m| m.box_id == *box_id && m.is_end)
            .cloned()
            .collect()
    })
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
                match transfer_from(award.clone(), ic_cdk::caller(), result_sub.clone()).await {
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
                            end_date: now + (MINER_TIME * 1_000_000_000),
                            is_end: false
                        };                                                 
                        MINERS.with(|miners: &std::cell::RefCell<BTreeMap<String, T::Miner>>| {
                            miners.borrow_mut().insert(new_canister_id.clone(), new_miner_info.clone());
                        });
                        BOX_MINER.with(|map| {
                            map.borrow_mut().insert(new_canister_id.clone(), box_id.clone());
                        });
                        let canister_id_for_miner = new_canister_id.clone();
                        //---------TIMER
                        print(format!("Starting miner timer {:?}", new_canister_id.clone()));
                        set_timer(Duration::from_secs(MINER_TIME), move || {                            
                            ic_cdk::spawn(async move {
                                print(format!("Miner {:?} is over", canister_id_for_miner.clone()));

                                MINERS.with(|miners_ref| {
                                    let mut miners = miners_ref.borrow_mut();
                                    if let Some(miner) = miners.get_mut(&canister_id_for_miner) {
                                        miner.is_end = true;
                                    }
                                });

                                let prize_pool = award.clone() * 25u32 / 100u32;
                                let for_box_creator = award.clone() * 65u32 / 100u32;
                                let admin_tax = award.clone() - prize_pool.clone() - for_box_creator.clone();   

                                let is_end = BOXES.with(|boxes_ref| {
                                    let boxes = boxes_ref.borrow();
                                    if let Some(box_id) = boxes.get(&canister_id_for_miner) {                                        
                                        box_id.is_end
                                    }
                                    else {
                                        true
                                    }
                                });         
                                if is_end {
                                    match transfer(award - FEE, result_sub.clone(), api::id(), None).await {
                                        Ok(index) => {
                                            print(format!("WHOLE_admin_tax success: {:?}", index));
                                        },
                                        Err(e) => {
                                            print(format!("WHOLE_admin_tax failed: {:?}", e));
                                        }
                                    }
                                }   
                                else {                                                                                                                                          
                                    match transfer(admin_tax - FEE, result_sub.clone(), api::id(), None).await {
                                        Ok(index) => {
                                            print(format!("admin_tax success: {:?}", index));
                                        },
                                        Err(e) => {
                                            print(format!("admin_tax failed: {:?}", e));
                                        }
                                    }
                                    
                                    match transfer(for_box_creator - FEE, result_sub.clone(), ic_cdk::caller(), None).await {
                                        Ok(index) => {
                                            print(format!("for_box_creator success: {:?}", index));
                                        },
                                        Err(e) => {
                                            print(format!("for_box_creator failed: {:?}", e));
                                        }
                                    }
                                                                
                                    match call::<(), (Vec<u8>,)>(candid::Principal::from_text(box_id).unwrap(), "get_subaccount", ()).await {
                                        Ok((sub_vec,)) => {
                                            let sub = Some(sub_vec); 
                                            match transfer(prize_pool - FEE, result_sub.clone(), api::id(), sub).await {
                                                Ok(index) => {
                                                    print(format!("prize_pool success: {:?}", index));
                                                },
                                                Err(e) => {
                                                    print(format!("prize_pool failed: {:?}", e));
                                                }
                                            }
                                        }
                                        Err(e) => {print(format!("get_subaccount for prize_pool failed: {}", e.1));}
                                    } 
                                }
                                
                            });                                                    
                        });               
                        //--------------
                       
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
async fn create_box(award: Nat) -> Result<T::BoxWithCount, String> {        
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
                            end_date: now + (LOTTERY_TIME * 1_000_000_000),
                            is_end: false
                        };    
                        BOXES.with(|boxes: &std::cell::RefCell<BTreeMap<String, T::BoxInfo>>| {
                            boxes.borrow_mut().insert(new_canister_id.clone(), new_box_info.clone());
                        });
                        let result_info = call_get_principal(new_canister_id.clone()).await;
                        //---------TIMER
                        let canister_id_for_lottery = new_canister_id.clone();
                        print(format!("Starting lottery timer {:?}", new_canister_id.clone()));
                        set_timer(Duration::from_secs(LOTTERY_TIME), move || {                            
                            ic_cdk::spawn(async move {
                               print(format!("Lottery {:?} is over", canister_id_for_lottery.clone()));
                               BOXES.with(|boxes_ref| {
                                let mut boxes = boxes_ref.borrow_mut();
                                if let Some(box_i) = boxes.get_mut(&canister_id_for_lottery.clone()) {
                                    box_i.is_end = true;
                                }
                                });
                               let winner = choose_random_miner(canister_id_for_lottery.clone()).await;
                               if winner.is_some()
                               {
                                    match get_balance(ic_cdk::caller(), None).await {
                                        Ok(balance64) => {                                        
                                            let balance_nat = Nat::from(balance64);                                    
                                            let prize: Nat = balance_nat - FEE;      
                                            match transfer(prize, result_sub.clone(), Principal::from_text(winner.unwrap().user).unwrap(), None).await {
                                                Ok(index) => {
                                                    print(format!("Prize success: {:?}", index));
                                                },
                                                Err(e) => {
                                                    print(format!("Prize failed: {:?}", e));
                                                }
                                            }
                                        },
                                        Err(e) => { print(format!("Prize get balance failed: {:?}", e)); }
                                        } 
                                }
                               else {
                                    print(format!("NO miners in {:?} ", canister_id_for_lottery.clone()));
                                    match get_balance(ic_cdk::caller(), None).await {
                                        Ok(balance64) => {                                        
                                            let balance_nat = Nat::from(balance64);                                    
                                            let prize: Nat = balance_nat - FEE;      
                                            match transfer(prize, result_sub.clone(), ic_cdk::caller(), None).await {
                                                Ok(index) => {
                                                    print(format!("return Prize success: {:?}", index));
                                                },
                                                Err(e) => {
                                                    print(format!("return Prize failed: {:?}", e));
                                                }
                                            }
                                        },
                                        Err(e) => { print(format!("Prize get balance failed: {:?}", e)); }
                                    } 
                               }
                              
                            });                                                    
                        });               
                        //--------------
                        let maybe_username = get_user_by_princ(new_box_info.clone().user);
                        let username: String = match maybe_username {
                            Some(user) => user.nickname,
                            None => "Unknown".to_string(),
                        };
                        let answer = T::BoxWithCount {
                            username: username,
                            miner_count: 0,
                            end_date: new_box_info.clone().end_date,
                            reg_date: new_box_info.clone().reg_date,
                            canister_id: new_box_info.clone().canister_id,
                        };
                        Ok(answer)
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

async fn choose_random_miner(box_id: String) -> Option<T::Miner> {
    let filtered_miners = get_active_miners(box_id);
    if filtered_miners.is_empty() {
        return None;
    }

    // Get 32 bytes of randomness from the IC
    let (random_bytes,): (Vec<u8>,) = raw_rand().await.unwrap();

    // Convert the first 8 bytes to a u64
    let rand_num = u64::from_le_bytes(random_bytes[0..8].try_into().unwrap());
    let idx = (rand_num as usize) % filtered_miners.len();

    Some(filtered_miners[idx].clone())
}

async fn transfer(amount: Nat, from_sub: Option<Vec<u8>>, to: Principal, to_sub: Option<Vec<u8>>,) -> Result<Nat, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {        
            let transfer_args = T::TransferArg {
                memo: None,
                amount: amount,
                from_subaccount: from_sub,
                fee: None,
                to: T::ICRCAccount { owner: to, subaccount: to_sub },
                created_at_time: None,
            };

            match call::<(T::TransferArg,), (T::TranferResult,)>(ledger_principal, "icrc1_transfer", (transfer_args,)).await {
                Ok((result,)) => {
                    match result {
                        T::TranferResult::Ok(index) => {
                            Ok(index)
                        },
                        T::TranferResult::Err(e) => {
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
                Err(e) => Err(format!("icrc1_transfer Call failed: {:?}", e)),
            }
        }
        Err(err) => {
            Err("Bad ledger canister".to_string())
        }
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