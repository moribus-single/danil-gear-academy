#![no_std]

use gstd::{prelude::*, ActorId};

#[derive(Encode, Decode, TypeInfo)]
pub struct InitEscrow {
    pub seller: ActorId,
    pub buyer: ActorId,
    pub price: u128,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum EscrowAction {
    Deposit(ActorId),
    ConfirmDelivery(ActorId),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum EscrowEvent {
    ProgramInitialized,
    FundsDeposited,
    DeliveryConfirmed,
    PaymentToSeller,
}
