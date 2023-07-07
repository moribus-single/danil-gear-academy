use gtest::{Log, Program, System};
use hello_world_io::*;

const OWNER: u64 = 100;
const USER: u64 = 101;
const USER2: u64 = 102;
const PROGRAM_ID: u64 = 1;

fn init_tamagotchi(sys: &System) {
    sys.init_logger();
    let program = Program::current(&sys);
    let res = program.send_bytes(OWNER, String::from("Satoshi"));
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(String::from("Success!"));

    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
    assert!(res.contains(&expected_log));
}

#[test]
fn tamagotchi_name() {
    // initialize env
    let sys = System::new();

    init_tamagotchi(&sys);
    let program = sys.get_program(PROGRAM_ID);

    let actor_id = OWNER;

    // test for TmgAction::Name
    let res = program.send(actor_id, TmgAction::Name);

    let expected_log = Log::builder()
        .dest(actor_id)
        .payload(TmgEvent::Name(String::from("Satoshi")));

    assert!(res.contains(&expected_log));
    assert!(!res.main_failed());
}
#[test]
fn tamagotchi_mood() {
    // initialize env
    let sys = System::new();

    // initialize a contract, get program by id
    init_tamagotchi(&sys);
    let program = sys.get_program(PROGRAM_ID);

    // read the state
    let state: Tamagotchi = program.read_state().expect("Error while reading the state");

    // check the fed value before feeding
    assert!(state.fed == 500, "Invalid fed value");


    // feed the tamagotchi
    let res = program.send(OWNER, TmgAction::Feed);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Fed);
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // play with tamagotchi
    let res = program.send(OWNER, TmgAction::Play);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Entertained);
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // rest with tamagotchi
    let res = program.send(OWNER, TmgAction::Sleep);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Slept);
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));


    // read the state
    // assert state is changed
    let state: Tamagotchi = program.read_state().expect("Error while reading the state");
    assert!(state.fed == 500 + FILL_PER_FEED, "Invalid fed value");
    assert!(state.entertained == 500 + FILL_PER_ENTERTAINMENT, "Invalid happy value");
    assert!(state.rested == 500 + FILL_PER_SLEEP, "Invalid happy value");
}

#[test]
fn tamagotchi_transfer() {
    // initialize env
    let sys = System::new();

    // initialize a contract, get program by id
    init_tamagotchi(&sys);
    let program = sys.get_program(PROGRAM_ID);

    let from = 32;
    // must fail since `from` is not OWNER
    let res = program.send(from, TmgAction::Transfer(USER.into()));
    assert!(res.main_failed());

    // successful transfer
    let res = program.send(OWNER, TmgAction::Transfer(USER.into()));
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Transfer(USER.into()));
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // check the state
    let state: Tamagotchi = program.read_state().expect("Failed to read the state");
    assert!(state.owner == USER.into());
}

#[test]
fn tamagotchi_approve() {
    // initialize env
    let sys = System::new();

    // initialize a contract, get program by id
    init_tamagotchi(&sys);
    let program = sys.get_program(PROGRAM_ID);

    let from = 32;

    // must fail since `from` is not owner
    let res = program.send(from, TmgAction::Approve(USER.into()));
    assert!(res.main_failed());

    // successful approve
    let res = program.send(OWNER, TmgAction::Approve(USER.into()));
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Approve(USER.into()));
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // check the state after approval
    let state: Tamagotchi = program.read_state().expect("Failed to read the state");
    assert!(state.allowed_account.unwrap() == USER.into());

    // successful transfer from user
    let res = program.send(USER, TmgAction::Transfer(USER.into()));
    let expected_log = Log::builder()
        .dest(USER)
        .payload(TmgEvent::Transfer(USER.into()));
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // check the state after transfer
    let state: Tamagotchi = program.read_state().expect("Failed to read the state");
    assert!(state.owner == USER.into());
}

#[test]
fn tamagotchi_revoke_approval() {
    // initialize env
    let sys = System::new();

    // initialize a contract, get program by id
    init_tamagotchi(&sys);
    let program = sys.get_program(PROGRAM_ID);

    // approve user to transfer ownership
    let res = program.send(OWNER, TmgAction::Approve(USER.into()));
    assert!(!res.main_failed());

    // must fail since user2 is not owner
    let res = program.send(USER2, TmgAction::RevokeApproval);
    assert!(res.main_failed());

    let state: Tamagotchi = program.read_state().expect("Failed to read the state");
    assert!(state.allowed_account.unwrap() == USER.into());

    let res = program.send(OWNER, TmgAction::RevokeApproval);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::RevokeApproval);
    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
    assert!(res.contains(&expected_log));

    let state: Tamagotchi = program.read_state().expect("Failed to read the state");
    assert!(state.allowed_account.unwrap_or_default() != USER.into());
}
