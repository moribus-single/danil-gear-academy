#![no_std]
use gstd::{ActorId, msg, prelude::*, exec::{block_timestamp, block_height}};
use ft_main_io::{FTokenAction, FTokenEvent, LogicAction};
use store_io::{StoreAction, StoreEvent};
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
   pub transaction_id: TransactionId,
   pub ft_contract_id: ActorId,
}

impl  Tamagotchi {
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
        self.assert_admin();
        self.allowed_account = Some(actor_id);

        msg::reply(
            TmgEvent::Approve(actor_id),
            0
        ).expect("Failed to share the TmgEvent");
    }

    fn revoke_approval(&mut self) {
        self.assert_admin();
        self.allowed_account = None;

        msg::reply(
            TmgEvent::RevokeApproval,
            0
        ).expect("Failed to share the TmgEvent");
    }

    async fn approve_tokens(&mut self, account: &ActorId, amount: u128) {
        self.assert_admin();
        
        let result = msg::send_for_reply_as::<_, FTokenEvent>(
            self.ft_contract_id,
            FTokenAction::Message {
                transaction_id: self.transaction_id,
                payload: LogicAction::Approve {
                    approved_account: *account,
                    amount,
                },
            },
            0,
        )
        .expect("Error in sending a message `FTokenAction::Message`")
        .await;
        
        match result {
            Ok(FTokenEvent::Ok) => {
                let _ = self.transaction_id.wrapping_add(1);
                msg::reply(
                    TmgEvent::ApproveTokens{account: *account, amount},
                    0
                ).expect("Failed to share the TmgEvent");
            },
            _ => {
                msg::reply(
                    TmgEvent::ApprovalError,
                    0
                ).expect("Failed to share TmgEvent");
            },
        }
    }
    
    async fn buy_attribute(
        &mut self, 
        store_id: &ActorId, 
        attribute_id: AttributeId
    ) {
        self.assert_admin();
    
        let result = msg::send_for_reply_as::<_, StoreEvent>(
            *store_id,
            StoreAction::BuyAttribute {
                attribute_id
            },
            0
        )
        .expect("Error in sending a message `StoreAction::BuyAttribute`")
        .await;

        match result {
            Ok(StoreEvent::CompletePrevTx{attribute_id}) => {
                msg::reply(
                    TmgEvent::CompletePrevPurchase(attribute_id),
                    0
                ).expect("Failed to share TmgEvent");
            },
            Ok(StoreEvent::AttributeSold{success: true}) => {
                msg::reply(
                    TmgEvent::AttributeBought(attribute_id),
                    0
                ).expect("Failed to share TmgEvent");
            },
            _ => {
                msg::reply(
                    TmgEvent::ErrorDuringPurchase,
                    0
                ).expect("Failed to share TmgEvent");
            }
        }
    }

    fn set_ft_contract(&mut self, actor_id: &ActorId) {
        self.assert_admin();

        let tamagotchi = unsafe {
            TAMAGOTCHI.as_mut().expect("The contract is not initialized")
        };
        tamagotchi.ft_contract_id = *actor_id;

        msg::reply(
            TmgEvent::SetFTokenContract,
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

    fn assert_admin(&mut self) {
        assert_eq!(msg::source(), self.owner, "Only admin can send that message!")
    }
}

#[gstd::async_main]
async fn main() {
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
        TmgAction::Transfer(actor_id) => tamagotchi.transfer(actor_id),
        TmgAction::ApproveTokens {
            account, 
            amount
        } => tamagotchi.approve_tokens(&account, amount).await,
        TmgAction::BuyAttribute{
            store_id,
            attribute_id,
        } => tamagotchi.buy_attribute(&store_id, attribute_id).await,
        TmgAction::SetFTokenContract(actor_id) => tamagotchi.set_ft_contract(&actor_id)
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
    let ft_contract_id: ActorId = ActorId::zero();
    let transaction_id: u64 = 0;

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
            allowed_account,
            ft_contract_id,
            transaction_id
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