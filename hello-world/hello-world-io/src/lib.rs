#![no_std]

use codec::{Decode, Encode};
use gmeta::{InOut, Metadata};
use scale_info::TypeInfo;
use gstd::{prelude::*, ActorId};

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
   type Init = InOut<String,()>;
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
   Sleep,
   Feed,
   Play
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmgEvent {
   Name(String),
   Age(u64),
   Sleep(u16),
   Feed(u16),
   Play(u16)
}

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

pub const HUNGER_PER_BLOCK: u32 = 1;
pub const ENERGY_PER_BLOCK: u32 = 2;
pub const BOREDOM_PER_BLOCK: u32 = 2;

pub const FILL_PER_SLEEP: u16 = 1000;
pub const FILL_PER_FEED: u16 = 1000;
pub const FILL_PER_ENTERTAINMENT: u16 = 1000;

pub const MAX_FED: u16 = 10000;
pub const MAX_HAPPY: u16 = 10000;
pub const MAX_RESTED: u16 = 10000;

