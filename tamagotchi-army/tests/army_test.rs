use gtest::{Program, System};
use tamagotchi_io::*;

const PROGRAM_ID: u64 = 1;
const OWNER: u64 = 100;
const BUYER_1: u64 = 101;
const BUYER_2: u64 = 102;

fn init_tmg_army(sys: &System) {
    let tmg_code_id = sys.submit_code("./hello-world/target/wasm32-unknown-unknown/debug/hello_world.opt.wasm");
    println!("tmg_code_id = {}", tmg_code_id);
    let tmg_factory = Program::current(&sys);
    let res = tmg_factory.send(OWNER, tmg_code_id);

    assert!(!res.main_failed());
    assert!(res.log().is_empty());
}

#[test]
fn create_tamagotchi() {
    // initialize env
    let sys = System::new();

    init_tmg_army(&sys);
    let program = sys.get_program(PROGRAM_ID);

    // create tamagotchi #1
    let mut name = "tmg1".to_string();
    let mut res = program.send(BUYER_1, ArmyAction::CreateTamagotchi(name));
    let mut state: TmgArmy = program.read_state().expect("Error while reading the state");
    
    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
    assert!(state.tmg_number == 1);

    // create tamagotchi #2
    name = "tmg2".to_string();
    res = program.send(BUYER_2, ArmyAction::CreateTamagotchi(name));
    state = program.read_state().expect("Error while reading the state");

    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
    assert!(state.tmg_number == 2);
}