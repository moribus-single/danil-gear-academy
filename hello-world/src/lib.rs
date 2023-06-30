#![no_std]
use async_trait::async_trait;
use gstd::{ActorId, msg, prelude::*, ReservationId, exec::{block_timestamp, block_height, program_id, system_reserve_gas}};
use ft_main_io::{FTokenAction, FTokenEvent, LogicAction};
use store_io::{StoreAction, StoreEvent};
use hello_world_io::*;

const DELAY: u32 = 120;
const MIN_ATTRIBUTE: u64 = 400;
const INIT_ATTRIBUTE: u64 = 300;

static mut TAMAGOTCHI: Option<Tamagotchi> = None;

#[async_trait]
trait NFTamagotchi {
    fn transfer(&mut self, actor_id: ActorId);
    fn approve(&mut self, actor_id: ActorId);
    fn revoke_approval(&mut self);
    async fn approve_tokens(&mut self, account: &ActorId, amount: u128);
    async fn buy_attribute(
        &mut self, 
        store_id: &ActorId, 
        attribute_id: AttributeId
    );
    fn check_attributes(&mut self);
    fn set_ft_contract(&mut self, actor_id: &ActorId);
    fn feed(&mut self);
    fn calculate_curr_fed(&mut self) -> u64;
    fn play(&mut self);
    fn calculate_curr_entertained(&mut self) -> u64;
    fn sleep(&mut self);
    fn calculate_curr_rest(&mut self) -> u64;
    fn name(&mut self);
    fn age(&mut self);
    fn assert_admin(&mut self);
    fn reserve_gas(&mut self, reservation_amount: u64, duration: u32);
}

#[async_trait]
impl NFTamagotchi for Tamagotchi {
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

    fn check_attributes(&mut self) {
        // assert_eq!(msg::source(), program_id(), "Only contract can call this function");
        
        let curr_feed_level: u64 = self.calculate_curr_fed();
        let curr_entertain_level: u64 = self.calculate_curr_entertained();
        let curr_rest_level: u64 = self.calculate_curr_rest();

        // sending msgs to the owner
        if curr_feed_level < MIN_ATTRIBUTE {
            msg::send(
                self.owner,
                TmgEvent::FeedMe,
                0
            ).expect("Failed to share TmgEvent");
        }
        if curr_entertain_level < MIN_ATTRIBUTE {
            msg::send(
                self.owner,
                TmgEvent::PlayWithMe,
                0
            ).expect("Failed to share TmgEvent");
        }
        if curr_rest_level < MIN_ATTRIBUTE {
            msg::send(
                self.owner,
                TmgEvent::WantToSleep,
                0
            ).expect("Failed to share TmgEvent");
        } 

        // next state check
        msg::send_delayed(
            program_id(),
            TmgAction::CheckState,
            0,
            DELAY,
        ).expect("Error while sending delayed.");
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

        // calculating current hunger level
        let curr_feed_level: u64 = self.calculate_curr_fed();

        // updating the state
        self.fed = curr_feed_level + FILL_PER_FEED;
        self.fed_block = block_height() as u64;

        msg::reply(
            TmgEvent::Fed,
            0
        ).expect("Failed to share TmgEvent");
    }

    fn calculate_curr_fed(&mut self) -> u64 {
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

        return curr_feed_level;
    }

    fn play(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.entertained < 7000, "Tamagotchi entertained enough");

        // calculating current happy level
        let curr_happy_level = self.calculate_curr_entertained();

        // updating the state
        self.entertained = curr_happy_level + FILL_PER_ENTERTAINMENT;
        self.entertained_block = block_height() as u64;

        msg::reply(
            TmgEvent::Entertained,
            0
        ).expect("Failed to share TmgEvent");
    }

    fn calculate_curr_entertained(&mut self) -> u64 {
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

        return curr_happy_level;
    }

    fn sleep(&mut self) {
        assert!(msg::source() == self.owner, "Only owner can feed the tamagotchi");
        assert!(self.rested < 7000, "Tamagotchi don't wanna to sleep");

        // calculating current rested level
        let curr_rested_level = self.calculate_curr_rest();

        // updating the state
        self.rested = curr_rested_level + FILL_PER_SLEEP;
        self.rested_block = block_height() as u64;

        msg::reply(
            TmgEvent::Slept, 
            0
        ).expect("Failed to share TmgEvent");
    }

    fn calculate_curr_rest(&mut self) -> u64 {
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

        return curr_rested_level;
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

    fn reserve_gas(&mut self, reservation_amount: u64, duration: u32) {
        let reservation_id = ReservationId::reserve(
            reservation_amount,
            duration,
        ).expect("reservation across executions");
        self.reservations.push(reservation_id);

        msg::reply(
            TmgEvent::GasReserved,
            0
        ).expect("Failed to share TmgEvent");
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
        } => {
            reserve_gas();
            tamagotchi.approve_tokens(&account, amount).await;
        },
        TmgAction::BuyAttribute {
            store_id,
            attribute_id,
        } => {
            reserve_gas();
            tamagotchi.buy_attribute(&store_id, attribute_id).await;
        },
        TmgAction::SetFTokenContract(actor_id) => tamagotchi.set_ft_contract(&actor_id),
        TmgAction::CheckState => {
            reserve_gas();
            tamagotchi.check_attributes();
        },
        TmgAction::ReserveGas {
            reservation_amount,
            duration
        } => tamagotchi.reserve_gas(reservation_amount, duration),
    };
}

#[no_mangle]
extern "C" fn my_handle_signal() {
    let tamagotchi = unsafe {
        TAMAGOTCHI.get_or_insert(Default::default())
    };

    let reservation_id = if !tamagotchi.reservations.is_empty() {
        tamagotchi.reservations.remove(0)
    } else {
        return;
    };

    msg::send_from_reservation(
        reservation_id,
        tamagotchi.owner,
        TmgEvent::MakeReservation,
        0
    ).expect("Failed to share TmgEvent");
}

#[no_mangle]
extern "C" fn init() {
    let owner = msg::source();
    let name: String = String::from_utf8(
        msg::load_bytes().expect("Can't load tamagotchi name")
    ).expect("Can't decode tamagotchi name");
    let date_of_birth = block_timestamp();

    let fed = INIT_ATTRIBUTE;
    let entertained = INIT_ATTRIBUTE; 
    let rested = INIT_ATTRIBUTE;

    let fed_block = block_height() as u64;
    let entertained_block = block_height() as u64;
    let rested_block = block_height() as u64;
    let allowed_account: Option<ActorId> = None;
    let ft_contract_id: ActorId = ActorId::zero();
    let transaction_id: u64 = 0;
    let reservations: Vec<ReservationId> = Vec::new();

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
            transaction_id,
            reservations,
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

fn reserve_gas() {
    system_reserve_gas(1_000_000_000).expect("Error during system gas reservation");
}