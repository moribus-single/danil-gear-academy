#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
   type Init = InOut<InitEscrow, ()>;
   type Handle = InOut<EscrowAction, EscrowEvent>;
   type Reply = ();
   type Others = ();
   type Signal = ();
   type State = Escrow;
}

#[derive(Encode, Decode, TypeInfo)]
pub struct InitEscrow {
   pub seller: ActorId,
   pub buyer: ActorId,
   pub price: u128,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum EscrowAction {
   Deposit,
   ConfirmDelivery,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum EscrowEvent {
   FundsDeposited,
   DeliveryConfirmed,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum EscrowState {
   AwaitingPayment,
   AwaitingDelivery,
   Closed,
}

impl Default for EscrowState {
   fn default() -> Self {
       Self::AwaitingPayment
   }
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Escrow {
   pub seller: ActorId,
   pub buyer: ActorId,
   pub price: u128,
   pub state: EscrowState,
}
