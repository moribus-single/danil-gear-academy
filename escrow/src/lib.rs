#![no_std]
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use gstd::{msg, ActorId, prelude::*};
use escrow_io::{EscrowAction, EscrowEvent, EscrowState, InitEscrow};

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Escrow {
   pub seller: ActorId,
   pub buyer: ActorId,
   pub price: u128,
   pub state: EscrowState,
}

impl Escrow {
    fn deposit(&mut self) {
        assert_eq!(
            self.state,
            EscrowState::AwaitingPayment,
            "State must be `AwaitingPayment"
        );
        assert_eq!(
            msg::source(),
            self.buyer,
            "The message sender must be a buyer"
        );
        assert_eq!(
            msg::value(),
            self.price,
            "The attached value must be equal to set price"
        );

        self.state = EscrowState::AwaitingDelivery;
        msg::reply(EscrowEvent::FundsDeposited, 0)
            .expect("Error in reply `EscrowEvent::FundsDeposited");
    }
    fn confirm_delivery(&mut self) {
        assert_eq!(
            self.state,
            EscrowState::AwaitingDelivery,
            "State must be `AwaitingDelivery"
        );
        assert_eq!(
            msg::source(),
            self.buyer,
            "The message sender must be a buyer"
        );
    }
}

static mut ESCROW: Option<Escrow> = None;

#[no_mangle]
extern "C" fn handle () {
    let action: EscrowAction = msg::load().expect("Unable to decode `EscrowAction`");

    let escrow: &mut Escrow = unsafe { ESCROW.as_mut().expect("The contract is not initialized") };

    match action {
        EscrowAction::Deposit => escrow.deposit(),
        EscrowAction::ConfirmDelivery => escrow.confirm_delivery(),
    }
}

#[no_mangle]
extern "C" fn init () {
    let init_config: InitEscrow = msg::load().expect("Error in decoding `InitEscrow`");

    let escrow = Escrow {
        seller: init_config.seller,
        buyer: init_config.buyer,
        price: init_config.price,
        state: EscrowState::AwaitingPayment,
    };
    unsafe { ESCROW = Some(escrow) };
}

#[no_mangle]
extern "C" fn state() {
   let escrow = unsafe {
        ESCROW.get_or_insert(Default::default())
   };
   msg::reply(escrow, 0).expect("Failed to share state");
}

#[no_mangle]
extern "C" fn metahash() {
   let metahash: [u8;32] = include!("../.metahash");
   msg::reply(metahash, 0).expect("Failed to share metahash");
}
