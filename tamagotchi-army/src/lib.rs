#![no_std]
use gstd::{msg, prelude::*, prog::ProgramGenerator, ActorId, CodeId};
use async_trait::async_trait;
use tamagotchi_io::*;
use hello_world_io::*;

pub const GAS_FOR_CREATION: u64 = 2_500_000_000;

static mut TAMAGOTCHI_ARMY: Option<TmgArmy> = None;

#[async_trait]
pub trait Army {
    async fn create_tamagotchi(&mut self, owner: &ActorId, name: &String);
}

#[async_trait]
impl Army for TmgArmy {
    async fn create_tamagotchi(&mut self, owner: &ActorId, name: &String) {
        // deploy tamagotchi
        let (address, _) = ProgramGenerator::create_program_with_gas_for_reply(
            self.tmg_code_id,
            name,
            GAS_FOR_CREATION,
            0
        )
        .expect("Error during tamagotchi initialization")
        .await
        .expect("Program was not initialized");

        // sending transfer event
        let transfer_to: ActorId = *owner;
        let result = msg::send_for_reply_as::<_, TmgEvent>(
            address,
            TmgAction::Transfer(transfer_to),
            0,
        )
        .expect("Error in sending a message `TmgAction::Transfer`")
        .await;

        match result {
            Ok(TmgEvent::Transfer(_)) => {
                self.tmg_number = self.tmg_number.saturating_add(1);
                self.id_to_address.insert(self.tmg_number, address);

                msg::reply(
                    ArmyEvent::TamagotchiCreated {
                        tamagotchi_id: self.tmg_number,
                        tamagotchi_address: address,
                    },
                    0
                )
                .expect("Error during a reply `ArmyEvent::TamagotchiCreated`");
            },
            _ => {
                msg::reply(
                    ArmyEvent::TamagotchiNotTransfered(transfer_to),
                    0
                )
                .expect("Error during a reply `ArmyEvent::TamagotchiNotTransfered`");
            }
        }
    }
}

#[gstd::async_main]
async fn main() {
    let action: ArmyAction = msg::load().expect("Unable to decode `ArmyAction`");
    let factory = unsafe { TAMAGOTCHI_ARMY.get_or_insert(Default::default()) };
    
    let sender = msg::source();
    match action {
        ArmyAction::CreateTamagotchi(name) => factory.create_tamagotchi(&sender, &name).await,
    }
}

#[no_mangle]
extern "C" fn init() {
    let tmg_code_id: CodeId =
        msg::load().expect("Unable to decode CodeId of the Tamagotchi program");
    let tamagotchi_army = TmgArmy {
        tmg_code_id,
        ..Default::default()
    };
    unsafe { TAMAGOTCHI_ARMY = Some(tamagotchi_army) };
}

#[no_mangle]
extern "C" fn state() {
   let tamagotchi_army = unsafe {
    TAMAGOTCHI_ARMY.as_ref().expect("The contract is not initialized")
   };

   msg::reply(tamagotchi_army, 0).expect("Failed to share state");
}

#[no_mangle]
// It returns the Hash of metadata.
// .metahash is generating automatically while you are using build.rs
extern "C" fn metahash() {
   let metahash: [u8; 32] = include!("../.metahash");
   msg::reply(metahash, 0).expect("Failed to share metahash");
}