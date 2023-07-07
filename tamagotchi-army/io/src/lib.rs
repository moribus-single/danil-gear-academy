#![no_std]
use codec::{Decode, Encode};
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId, CodeId};
use scale_info::TypeInfo;

pub type TamagotchiId = u64;

pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = In<CodeId>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type Handle = InOut<ArmyAction, ArmyEvent>;
    type State = TmgArmy;
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct TmgArmy {
    pub tmg_number: TamagotchiId,
    pub id_to_address: BTreeMap<TamagotchiId, ActorId>,
    pub tmg_code_id: CodeId,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum ArmyAction {
    CreateTamagotchi(String),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum ArmyEvent {
    TamagotchiCreated {
        tamagotchi_id: TamagotchiId,
        tamagotchi_address: ActorId,
    },
    TamagotchiNotTransfered(ActorId),
}