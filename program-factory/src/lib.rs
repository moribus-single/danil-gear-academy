#![no_std]
use escrow_io::*;
use gstd::{msg, prelude::*, prog::ProgramGenerator, ActorId, CodeId};

pub const GAS_FOR_CREATION: u64 = 2_500_000_000;
pub type EscrowId = u64;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct EscrowFactory {
    pub escrow_number: EscrowId,
    pub id_to_address: BTreeMap<EscrowId, ActorId>,
    pub escrow_code_id: CodeId,
}
static mut ESCROW_FACTORY: Option<EscrowFactory> = None;

impl EscrowFactory {
    async fn create_escrow(&mut self, seller: &ActorId, buyer: &ActorId, price: u128) {
        let (address, _) = ProgramGenerator::create_program_with_gas_for_reply(
            self.escrow_code_id,
            InitEscrow {
                seller: *seller,
                buyer: *buyer,
                price,
            }
            .encode(),
            GAS_FOR_CREATION,
            0,
        )
        .expect("Error during Escrow program initialization")
        .await
        .expect("Program was not initialized");
    
        self.escrow_number = self.escrow_number.saturating_add(1);
        self.id_to_address.insert(self.escrow_number, address);
        msg::reply(
            FactoryEvent::EscrowCreated {
                escrow_id: self.escrow_number,
                escrow_address: address,
            },
            0,
        )
        .expect("Error during a reply `FactoryEvent::ProgramCreated`");
    }
    async fn deposit(&self, escrow_id: EscrowId) {
        let escrow_address = self.get_escrow_address(escrow_id);
        send_message(&escrow_address, EscrowAction::Deposit(msg::source())).await;

        msg::reply(FactoryEvent::Deposited(escrow_id), 0)
            .expect("Error during a reply `FactoryEvent::Deposited`");
    }
    async fn confirm_delivery(&self, escrow_id: EscrowId) {
        let escrow_address = self.get_escrow_address(escrow_id);
        send_message(
            &escrow_address,
            EscrowAction::ConfirmDelivery(msg::source()),
        )
        .await;
        msg::reply(FactoryEvent::DeliveryConfirmed(escrow_id), 0)
            .expect("Error during a reply `FactoryEvent::DeliveryConfirmed`");
    }

    fn get_escrow_address(&self, escrow_id: EscrowId) -> ActorId {
        *self
            .id_to_address
            .get(&escrow_id)
            .expect("The escrow with indicated id does not exist")
    }
}

#[gstd::async_main]
async fn main() {
    let action: FactoryAction = msg::load().expect("Unable to decode `FactoryAction`");
    let factory = unsafe { ESCROW_FACTORY.get_or_insert(Default::default()) };
    
    match action {
        FactoryAction::CreateEscrow {
            seller,
            buyer,
            price,
        } => factory.create_escrow(&seller, &buyer, price).await,
        FactoryAction::Deposit(escrow_id) => factory.deposit(escrow_id).await,
        FactoryAction::ConfirmDelivery(escrow_id) => factory.confirm_delivery(escrow_id).await,
    }
}

#[no_mangle]
extern "C" fn init() {
    let escrow_code_id: CodeId =
        msg::load().expect("Unable to decode CodeId of the Escrow program");
    let escrow_factory = EscrowFactory {
        escrow_code_id,
        ..Default::default()
    };
    unsafe { ESCROW_FACTORY = Some(escrow_factory) };
}

#[derive(Encode, Decode, TypeInfo)]
pub enum FactoryAction {
    CreateEscrow {
        seller: ActorId,
        buyer: ActorId,
        price: u128,
    },
    Deposit(EscrowId),
    ConfirmDelivery(EscrowId),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum FactoryEvent {
    EscrowCreated {
        escrow_id: EscrowId,
        escrow_address: ActorId,
    },
    Deposited(EscrowId),
    DeliveryConfirmed(EscrowId),
}

async fn send_message(escrow_address: &ActorId, escrow_payload: EscrowAction) {
    msg::send_for_reply_as::<_, EscrowEvent>(*escrow_address, escrow_payload, msg::value())
        .expect("Error during a sending message to a Escrow program")
        .await
        .expect("Unable to decode EscrowEvent");
}
