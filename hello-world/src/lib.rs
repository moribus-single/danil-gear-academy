#![no_std]
use gstd::{ActorId, msg, prelude::*, exec::{block_timestamp, block_height}};
use hello_world_io::*;

static mut TAMAGOTCHI: Option<Tamagotchi> = None;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Tamagotchi {
   pub owner: ActorId,
   pub name: String,
   pub date_of_birth: u64,

   pub fed: u16,
   pub happy: u16,
   pub rested: u16,

   pub last_ate: u32,
   pub last_had_fun: u32,
   pub last_slept: u32
}

impl Tamagotchi {
    fn feed(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.fed < 7000, "Tamagotchi not enough hungry");

        // calculating and normalizing hunger level
        let hunger_level: u32 = (block_height() - self.last_ate) * HUNGER_PER_BLOCK;
        let normalized_hunger_level: u16 = if hunger_level > MAX_FED {
            MAX_FED
        } else {
            hunger_level as u16
        };
        
        // calculating current hunger level
        let curr_feed_level = if self.fed > normalized_hunger_level {
            self.fed - normalized_hunger_level
        } else {
            0
        };

        // updating the state
        self.fed = curr_feed_level + FILL_PER_FEED;
        self.last_ate = block_height();

        msg::reply(
            TmgEvent::Feed(self.fed),
            0
        ).expect("Failed to share TmgEvent");
    }

    fn play(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.happy < 7000, "Tamagotchi entertained enough");

        // calculating and normalizing bored level
        let bored_level: u32 = (block_height() - self.last_had_fun) * BOREDOM_PER_BLOCK;
        let normalized_bored_level: u16 = if bored_level > MAX_HAPPY {
            MAX_HAPPY
        } else {
            bored_level as u16
        };

        // calculating current happy level
        let curr_happy_level = if self.happy > normalized_bored_level {
            self.happy - normalized_bored_level
        } else {
            0
        };

        // updating the state
        self.happy = curr_happy_level + FILL_PER_ENTERTAINMENT;
        self.last_had_fun = block_height();

        msg::reply(
            TmgEvent::Play(self.happy),
            0
        ).expect("Failed to share TmgEvent");
    }

    fn sleep(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.rested < 7000, "Tamagotchi don't wanna to sleep");

        // calculating and normalizing energy loss
        let energy_loss: u32 = (block_height() - self.last_slept) * ENERGY_PER_BLOCK;
        let normalized_energy_loss: u16 = if energy_loss > MAX_RESTED {
            MAX_RESTED
        } else {
            energy_loss as u16
        };

        // calculating current rested level
        let curr_rested_level = if self.rested > normalized_energy_loss {
            self.rested - normalized_energy_loss
        } else {
            0
        };

        // updating the state
        self.rested = curr_rested_level + FILL_PER_SLEEP;
        self.last_slept = block_height();

        msg::reply(
            TmgEvent::Sleep(self.rested), 
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
    let age: u64 = block_timestamp() - tamagotchi.date_of_birth;

    // loading TmgAction
    let action: TmgAction = msg::load().expect("Error in loading TmgAction");

    // matching pattern
    match action {
        TmgAction::Age => tamagotchi.age(),
        TmgAction::Name => tamagotchi.name(),
        TmgAction::Feed => tamagotchi.feed(),
        TmgAction::Play => tamagotchi.play(),
        TmgAction::Sleep => tamagotchi.sleep()
    };
}

#[no_mangle]
extern "C" fn init() {
    let owner = msg::source();
    let name = String::from_utf8(
        msg::load_bytes().expect("Can't load tamagotchi name")
    ).expect("Can't decode to String");
    let date_of_birth = block_timestamp();

    let fed = 500;
    let happy = 500;
    let rested = 500;

    let last_ate = block_height();
    let last_had_fun = block_height();
    let last_slept = block_height();


    unsafe { 
        TAMAGOTCHI = Some(Tamagotchi{
            owner,
            name,
            date_of_birth,
            fed,
            happy,
            rested,
            last_ate, // last_ate
            last_had_fun,
            last_slept
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