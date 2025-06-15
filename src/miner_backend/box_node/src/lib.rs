use types as T;
use ic_cdk::api;
use ic_cdk::api::call::call;

thread_local! {
   static SUB_ACCOUNT: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());
}
async fn get_balance() -> Result<u64, String> {
    let maybe_principal = candid::Principal::from_text(T::LEDGER_CANISTER);
    match maybe_principal {
        Ok(ledger_principal) => {
            let args = T::ICRCAccount {
                owner: api::id(), 
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


#[ic_cdk::update]
async fn init(sub: Vec<u8>) -> () {
    SUB_ACCOUNT.with(|s| {
        *s.borrow_mut() = sub;
    });
}


#[ic_cdk::update]
async fn get_principal() -> String {
    match get_balance().await {
        Ok(balance64) => {          
            let result = SUB_ACCOUNT.with(|sub| {
                let sub_bytes = sub.borrow();
                let sub_string = format!("{:?}", *sub_bytes); 
                format!("{} {} {}", api::id(), balance64, sub_string)
            });  
            return api::id().to_string() + " " + &balance64.to_string() + &result;
        },
        Err(e) => e,
    }
}