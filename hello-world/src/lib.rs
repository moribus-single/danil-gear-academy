#![no_std]
use gstd::{msg, prelude::*, exec::block_timestamp};

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Tamagotchi {
   pub name: String,
   pub date_of_birth: u64,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmgAction {
    Name,
    Age,
}
 
#[derive(Encode, Decode, TypeInfo)]
pub enum TmgEvent {
    Name(String),
    Age(u64),
}

static mut TAMAGOTCHI: Option<Tamagotchi> = None;

#[no_mangle]
extern "C" fn handle() {
    // prepairing data
    let tamagotchi = unsafe {
        TAMAGOTCHI.as_ref().expect("The contract is not initialized")
    };
    let age: u64 = block_timestamp() - tamagotchi.date_of_birth;

    // loading TmgAction
    let action: TmgAction = msg::load().expect("Error in loading TmgAction");

    // matching pattern
    let event: TmgEvent = match action {
        TmgAction::Age => TmgEvent::Age(age),
        TmgAction::Name => TmgEvent::Name(tamagotchi.name.clone())
    };

    // reply with TmgEvent
    msg::reply(event, 0).expect("Failed to share TmgEvent");
}

#[no_mangle]
extern "C" fn init() {
    let name = String::from_utf8(
        msg::load_bytes().expect("Can't load tamagotchi name")
    ).expect("Can't decode to String");
    let date_of_birth = block_timestamp();

    unsafe { 
        TAMAGOTCHI = Some(Tamagotchi{
            name,
            date_of_birth
        });
    };

    msg::reply(String::from("Success!"), 0).expect("Failed to share initialization result");
}

#[no_mangle]
extern "C" fn state() {
   let tamagotchi = unsafe {
        TAMAGOTCHI.as_ref().expect("The contract is not initialized")
   };

   msg::reply(tamagotchi, 0).expect("Failed to share state");
}

#[no_mangle]
// It returns the Hash of metadata.
// .metahash is generating automatically while you are using build.rs
extern "C" fn metahash() {
   let metahash: [u8; 32] = include!("../.metahash");
   msg::reply(metahash, 0).expect("Failed to share metahash");
}