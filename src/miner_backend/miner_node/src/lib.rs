use types as T;
use ic_cdk::api;
use ic_cdk::api::call::call;

thread_local! {
   static SUB_ACCOUNT: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::new());
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