use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult
};

// use crate::error::ContractError;
use crate::msg::{CardResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Card, CARD_VIEWING_KEY, ENTROPY, USER_CARDS,OWNER,ADMIN,MODIFY};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

use secret_toolkit::crypto::sha_256;
use secret_toolkit::crypto::Prng;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};
const STORAGE_KEY: &'static [u8] = b"viewing_keys";
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    ENTROPY.save(deps.storage, &msg.entropy)?;
    OWNER.save(deps.storage,&_info.sender)?;
    ADMIN.save(deps.storage,&_info.sender)?;
    MODIFY.save(deps.storage,&false)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Create { card, index } => try_create_card(deps, info, card, index),
        ExecuteMsg::Burn { index } => try_burn_card(deps, env, info, index),
        ExecuteMsg::GenerateViewingKey {index,reciever } => {
            try_generate_viewing_key(deps, env, info, index,reciever)
        }
        
        ExecuteMsg::DeleteKey {account} => {
        	delete_viewing_key(deps,env,info,account)
        
        
        }
        
         ExecuteMsg::PermissionWrite {wallet} => {
        	try_permission_write(deps,info,wallet)
        
        
        }
        
    }
}

pub fn try_permission_write(

 deps: DepsMut,
 info: MessageInfo,
 wallet:Addr
    
  

)->Result<Response, StdError>{


  ADMIN.save(deps.storage,&wallet)?;
  let admin_modify_status=MODIFY.load(deps.storage)?;
  if admin_modify_status == false {
  
      MODIFY.save(deps.storage,&true)?;
  
  }
  
  
  



 Ok(Response::default())


}



pub fn try_create_card(
    deps: DepsMut,
    info: MessageInfo,
    card: Card,
    index: u8,
) -> Result<Response, StdError> {
	let owner=OWNER.load(deps.storage)?;
	let admin=ADMIN.load(deps.storage)?;
	let admin_modify_status=MODIFY.load(deps.storage)?;
	
	if (owner==info.sender) || (admin==info.sender && admin_modify_status == true){
	
    USER_CARDS
        .add_suffix(owner.as_bytes())
        .insert(deps.storage, &index, &card)?;
    
  ADMIN.save(deps.storage,&owner)?;
  let admin_modify_status=MODIFY.load(deps.storage)?;
  if admin_modify_status == true {
  
      MODIFY.save(deps.storage,&false)?;
  
  }
    
    Ok(Response::default())
    }
    else{
    	Err(StdError::generic_err("Wrong sender, you don't authorized to make change"))
    }
    
    
}

pub fn try_burn_card(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    index: u8,
) -> Result<Response, StdError> {

let owner=OWNER.load(deps.storage)?;
	let admin=ADMIN.load(deps.storage)?;
	let admin_modify_status=MODIFY.load(deps.storage)?;
	
	if (owner==info.sender) || (admin==info.sender && admin_modify_status == true){


        USER_CARDS.add_suffix(info.sender.as_bytes()).remove(deps.storage,&index)?;
        Ok(Response::default())
       


    }
  
     

    
else{
Err(StdError::generic_err("Wrong sender, you don't authorized to make change"))
}

}

pub fn try_generate_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: u8,
    reciever:String
) -> StdResult<Response> {
    //map for viewing keys
    
   let owner=OWNER.load(deps.storage)?;
	let admin=ADMIN.load(deps.storage)?;
	let admin_modify_status=MODIFY.load(deps.storage)?;
	
	if (owner==info.sender) || (admin==info.sender && admin_modify_status == true){
    

    
    let info_ref=&info;
   
    ViewingKey::set_seed(deps.storage,  b"seed");
    let viewing_key = ViewingKey::create(deps.storage, info_ref, &env, reciever.as_str(), b"entropy");

    
  
    let res = Response::default().add_attribute("viewing_key", viewing_key);

    Ok(res)
}
else{

Err(StdError::generic_err("Wrong sender, you don't authorized to make change"))

}

}



pub fn delete_viewing_key( 
deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    account: String,
    )-> StdResult<Response>{
    
    
      //PrefixedStorage::new(deps.storage, STORAGE_KEY).remove(account.as_bytes());
      deps.storage.remove(account.as_bytes());
      
      Ok(Response::default())
    
    

    
    }

/* new_viewing_key is used to generate a unique, random key for each business card we create.The combination of the current time and the sender of the message is used as entropy to initialize the random number generator, so that each message has a unique viewing key that is derived from information specific to that message.
 */
pub fn new_viewing_key(env: &Env, info: MessageInfo, entropy_bytes: &[u8]) -> String {
    //1. The variable entropy_len is defined as the length of 16 + the length of the sender field in the "info" struct + the length of `entropy_bytes`.
    let entropy_len = 16 + info.sender.as_bytes().len() + entropy_bytes.len();
    //2. A Vec named rng_entropy is created with a capacity equal to entropy_len. The vector is then filled with entropy_bytes and the time (in nanoseconds) of the block stored in env.
    let mut rng_entropy = Vec::with_capacity(entropy_len);

    rng_entropy.extend_from_slice(&env.block.time.nanos().to_be_bytes());
    rng_entropy.extend_from_slice(info.sender.as_bytes());
    rng_entropy.extend_from_slice(entropy_bytes);
    // 3. A random number is created using Prng::new with the entropy "entropy_bytes" and the "rng_entropy" vector.
    let mut rng = Prng::new(entropy_bytes, &rng_entropy);
    //4. The method rng.rand_bytes generates a random slice of bytes.
    let rand_slice = rng.rand_bytes();
    //5. Then, we calculate the SHA-256 hash of the random slice, and store it in the "key" variable.
    let key = sha_256(&rand_slice);
    //6. Finally, we return the base64 encoding of the key as a String.
    // base64::encode is being depreciated, todo: switch to engine method instead
    base64::encode(key)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env,msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCardViewer {
            wallet,
            viewing_key,
            index,
            
        } => to_binary(&query_card_viewer(deps, wallet, viewing_key, index)?),
        
        
        /* QueryMsg::GetCardAllOwner {
          
          wallet,
          toindex
            
        } => to_binary(&query_card_owner_all(deps,wallet,toindex)?),*/
        
        
    }
}

/*fn query_card_owner_all(deps: Deps,wallet: String,toindex:u8) -> StdResult<CardResponse> {
	let owner=OWNER.load(deps.storage)?;
	if owner==wallet{
	let mut v1 : Vec <Card>=Vec ::new();
	
	let x=USER_CARDS.add_suffix(owner.as_bytes());
	
	
	let mut i=0;
	while i <= toindex{
		
	 let sample=x.get(deps.storage, &i);
	 match sample{
	 
	 Some(card) => {
            			v1.push(card);
            			},
         None => (),
        }
	
	}
	
	
	Ok(CardResponse { card: v1 })
		
	
	
	/*
	let mut y=x.iter(deps.storage)?;
	//let mut y=x.next();
	
	
	while x.next().is_some(){
		
	
		v1.push(x.(owner.as_bytes()));
		
		
		
	}
	
	
	loop {
	

	match y.next() {
	
	Some(Ok((_,card)))=>v1.push(card),
	Some(Err(_)) => return Err(StdError::generic_err("You are not authorized to read")),
	None => return Err(StdError::generic_err("You are not authorized to read")),
	
	
	}
	

	
	}
	
	
	
	
	Ok(CardResponse { card: v1 })
		
	
	
	}
	
	
	else{
	
	Err(StdError::generic_err("You are not authorized to read"))
	
	
	}
	
	*/
		}
		
		
		else{
		Err(StdError::generic_err("You are not authorized to read"))
		
		}



}*/

fn query_card_viewer(deps: Deps,wallet: Addr, viewing_key: String, index: u8) -> StdResult<CardResponse> {	

	let owner=OWNER.load(deps.storage)?;
	let admin=ADMIN.load(deps.storage)?;
	let admin_modify_status=MODIFY.load(deps.storage)?;
	
	if (owner==wallet) || (admin==wallet && admin_modify_status == true){
	
	let card_exists = USER_CARDS
		    .add_suffix(owner.as_bytes())
		    .get(deps.storage, &index);
		  match card_exists {
            		Some(card) => Ok(CardResponse { card: card }),
            		None => Err(StdError::generic_err("Card not here!")),
        }
            
	
		
	
	
	
	}else{


	

	let result = ViewingKey::check(deps.storage,wallet.as_str(), viewing_key.as_str());
	//let mut v1 : Vec <Card>=Vec ::new();
	if result.is_ok(){
		
	
		let card_exists = USER_CARDS
		    .add_suffix(owner.as_bytes())
		    .get(deps.storage, &index);
		  match card_exists {
            		Some(card) => {
            			//v1.push(card);
            			Ok(CardResponse { card: card })},
            		None => Err(StdError::generic_err("Card not here!")),
        }
            
	
	
	
	}
	
	else{
	
	Err(StdError::generic_err("Viewing Key Wrong"))
	}
	}
	

/*
    let viewing_keys_exists = CARD_VIEWING_KEY
        .add_suffix(wallet.as_bytes()).get(deps.storage,&index).unwrap();
      
        
    

    if viewing_keys_exists ==viewing_key {
    	
        let card_exists = USER_CARDS
            .add_suffix(owner.as_bytes())
            .get(deps.storage, &index);
        assert_eq!(USER_CARDS.get_len(deps.storage)?, 1);
            
        match card_exists {
            Some(card) => Ok(CardResponse { card: card }),
            None => Err(StdError::generic_err("Card not here!")),
        }
    } else {
        Err(StdError::generic_err("Wrong viewing key!"))
    }
    */
}



#[cfg(test)]

mod tests {

use cosmwasm_std::testing::*;
use serde::Deserialize;

use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,Storage
};


// use crate::error::ContractError;
use crate::msg::{CardResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Card, CARD_VIEWING_KEY, ENTROPY, USER_CARDS,OWNER,ADMIN,MODIFY};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use cosmwasm_std::from_binary;


use secret_toolkit::crypto::sha_256;
use secret_toolkit::crypto::Prng;
use secret_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};


   use super::*;

#[test]
fn testing(){

let account = "user-1".to_string();
let mut deps = mock_dependencies();
let env = mock_env();
let info = mock_info(account.as_str(), &[]);


  let init_msg = InstantiateMsg {
            entropy: "this ".to_string() 
        };
       let result= instantiate(deps.as_mut(),env,info,init_msg);
  assert_eq!(result, Ok(Response::default()));
  
  
  
  /*
  let Card={
  	name: "DElete me",
            address: "DElete me ",
            phone: "12345678953",
  };*/
  
  
  // create table 
  let mesg = ExecuteMsg :: Create { card: Card{
  	name: "DElete me".to_string(),
            address: "DElete me ".to_string(),
            phone: "12345678953".to_string(),
  }, index: 0 };
   

	
	
	let result1=execute(deps.as_mut(),mock_env(),mock_info(account.as_str(), &[]),mesg);
	
	assert_eq!(result1, Ok(Response::default()));
	
	
	
	
	
	
		
	
}
}





