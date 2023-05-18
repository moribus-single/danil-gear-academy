#![no_std]
use gstd::{ActorId, msg, prelude::*, exec::{block_timestamp, block_height}};
use hello_world_io::*;

static mut TAMAGOTCHI: Option<Tamagotchi> = None;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Tamagotchi {
   pub name: String,
   pub date_of_birth: u64,
   pub owner: ActorId,

   pub fed: u64,
   pub fed_block: u64,
   pub entertained: u64,
   pub entertained_block: u64,
   pub rested: u64,
   pub rested_block: u64,

   pub allowed_account: Option<ActorId>,
}

impl Tamagotchi {
    fn transfer(&mut self, actor_id: ActorId) {
        let sender = msg::source();
        assert!(
            sender == self.owner || self.allowed_account == Some(sender),
            "Only owner or allowed account can transfer ownership"
        );
        self.owner = actor_id;

        msg::reply(
            TmgEvent::Transfer(actor_id), 
            0
        ).expect("Failed to share the TmgEvent");
    }

    fn approve(&mut self, actor_id: ActorId) {
        assert!(msg::source() == self.owner, "Only owner can approve");
        self.allowed_account = Some(actor_id);

        msg::reply(
            TmgEvent::Approve(actor_id),
            0
        ).expect("Failed to share the TmgEvent");
    }

    fn revoke_approval(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can revoke approval");
        self.allowed_account = None;

        msg::reply(
            TmgEvent::RevokeApproval,
            0
        ).expect("Failed to share the TmgEvent");
    }

    fn feed(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.fed < 7000, "Tamagotchi not enough hungry");

        // calculating and normalizing hunger level
        let hunger_level = (block_height() as u64 - self.fed_block) * HUNGER_PER_BLOCK;
        let normalized_hunger_level = if hunger_level > MAX_FED {
            MAX_FED
        } else {
            hunger_level
        };
        
        // calculating current hunger level
        let curr_feed_level: u64 = if self.fed > normalized_hunger_level {
            self.fed - normalized_hunger_level
        } else {
            0
        };

        // updating the state
        self.fed = curr_feed_level + FILL_PER_FEED;
        self.fed_block = block_height() as u64;

        msg::reply(
            TmgEvent::Fed,
            0
        ).expect("Failed to share TmgEvent");
    }

    fn play(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.entertained < 7000, "Tamagotchi entertained enough");

        // calculating and normalizing bored level
        let bored_level: u64 = (block_height() as u64 - self.entertained_block) * BOREDOM_PER_BLOCK;
        let normalized_bored_level: u64 = if bored_level > MAX_HAPPY {
            MAX_HAPPY
        } else {
            bored_level as u64
        };

        // calculating current happy level
        let curr_happy_level = if self.entertained > normalized_bored_level {
            self.entertained - normalized_bored_level
        } else {
            0
        };

        // updating the state
        self.entertained = curr_happy_level + FILL_PER_ENTERTAINMENT;
        self.entertained_block = block_height() as u64;

        msg::reply(
            TmgEvent::Entertained,
            0
        ).expect("Failed to share TmgEvent");
    }

    fn sleep(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.rested < 7000, "Tamagotchi don't wanna to sleep");

        // calculating and normalizing energy loss
        let energy_loss: u64 = (block_height() as u64 - self.rested_block) * ENERGY_PER_BLOCK;
        let normalized_energy_loss: u64 = if energy_loss > MAX_RESTED {
            MAX_RESTED
        } else {
            energy_loss as u64
        };

        // calculating current rested level
        let curr_rested_level = if self.rested > normalized_energy_loss {
            self.rested - normalized_energy_loss
        } else {
            0
        };

        // updating the state
        self.rested = curr_rested_level + FILL_PER_SLEEP;
        self.rested_block = block_height() as u64;

        msg::reply(
            TmgEvent::Slept, 
            0
        ).expect("Failed to share TmgEvent");
    }

    fn name(&mut self) {
        msg::reply(
            TmgEvent::Name(self.name.clone()), 
            0
        ).expect("Failed to share the TmgEvent");
    }

    fn age(&mut self) {
        msg::reply(
            TmgEvent::Age(block_timestamp() - self.date_of_birth),
            0
        ).expect("Failed to share the TmgEvent");
    }
}

#[no_mangle]
extern "C" fn handle() {
    // prepairing data
    let tamagotchi = unsafe {
        TAMAGOTCHI.as_mut().expect("The contract is not initialized")
    };

    // loading TmgAction
    let action: TmgAction = msg::load().expect("Error in loading TmgAction");

    // matching pattern
    match action {
        TmgAction::Age => tamagotchi.age(),
        TmgAction::Name => tamagotchi.name(),
        TmgAction::Feed => tamagotchi.feed(),
        TmgAction::Play => tamagotchi.play(),
        TmgAction::Sleep => tamagotchi.sleep(),
        TmgAction::RevokeApproval => tamagotchi.revoke_approval(),
        TmgAction::Approve(actor_id) => tamagotchi.approve(actor_id),
        TmgAction::Transfer(actor_id) => tamagotchi.transfer(actor_id)
    };
}

#[no_mangle]
extern "C" fn init() {
    let owner = msg::source();
    let name: String = String::from_utf8(
        msg::load_bytes().expect("Can't load tamagotchi name")
    ).expect("Can't decode tamagotchi name");
    let date_of_birth = block_timestamp();

    let fed = 500;
    let entertained = 500; 
    let rested = 500;

    let fed_block = block_height() as u64;
    let entertained_block = block_height() as u64;
    let rested_block = block_height() as u64;
    let allowed_account: Option<ActorId> = None;

    unsafe { 
        TAMAGOTCHI = Some(Tamagotchi{
            name,
            date_of_birth,
            owner,
            fed,
            fed_block, 
            entertained,
            entertained_block,
            rested,
            rested_block,
            allowed_account
        });
    };

    msg::reply(String::from("Success!"), 0)
        .expect("Failed to share initialization result");
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