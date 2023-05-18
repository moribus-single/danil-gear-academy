#![no_std]

use codec::{Decode, Encode};
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
   type Init = In<String>;
   type Reply = ();
   type Others = ();
   type Signal = ();
   type Handle = InOut<TmgAction, TmgEvent>;
   type State = Tamagotchi;
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmgAction {
   Name,
   Age,
   Feed,
   Play,
   Sleep,
   Transfer(ActorId),
   Approve(ActorId),
   RevokeApproval,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmgEvent {
   Name(String),
   Age(u64),
   Fed,
   Entertained,
   Slept,
   Transfer(ActorId),
   Approve(ActorId),
   RevokeApproval,
}

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

pub const HUNGER_PER_BLOCK: u64 = 1;
pub const ENERGY_PER_BLOCK: u64 = 2;
pub const BOREDOM_PER_BLOCK: u64 = 2;

pub const FILL_PER_SLEEP: u64 = 1000;
pub const FILL_PER_FEED: u64 = 1000;
pub const FILL_PER_ENTERTAINMENT: u64 = 1000;

pub const MAX_FED: u64 = 10000;
pub const MAX_HAPPY: u64 = 10000;
pub const MAX_RESTED: u64 = 10000;

