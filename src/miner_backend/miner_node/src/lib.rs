use types as T;
use ic_cdk::api;
use std::cell::RefCell;
use std::time::Duration;

thread_local! {
    static SUB_ACCOUNT: RefCell<Vec<u8>> = RefCell::new(Vec::new());   

}

#[ic_cdk::update]
async fn init(sub: Vec<u8>) -> () {
    SUB_ACCOUNT.with(|s| {
        *s.borrow_mut() = sub;        
    });
}


#[ic_cdk::update]
async fn get_info() -> String {
    let result = SUB_ACCOUNT.with(|sub| {
        let sub_bytes = sub.borrow();
        let sub_string = format!("{:?}", *sub_bytes); 
        sub_string
    });  
    return api::id().to_string() + " MINER " + &result;
}

#[ic_cdk::update]
async fn get_subaccount() -> Vec<u8> {
    SUB_ACCOUNT.with(|sub| sub.borrow().to_vec())
}
