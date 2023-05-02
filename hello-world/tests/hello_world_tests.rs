use gtest::{Log, Program, System};
use hello_world_io::*;

const OWNER: u64 = 100;
const PROGRAM_ID: u64 = 1;

fn init_hello_world(sys: &System) {
    sys.init_logger();
    let program = Program::current(&sys);
    let res = program.send_bytes(OWNER, String::from("Satoshi"));
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(String::from("Success!"));

    assert!(!res.log().is_empty());
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));
}

#[test]
fn tamagotchi_name() {
    // initialize env
    let sys = System::new();

    init_hello_world(&sys);
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
    init_hello_world(&sys);
    let program = sys.get_program(PROGRAM_ID);

    // read the state
    let state: Tamagotchi = program.read_state().expect("Error while reading the state");

    // check the fed value before feeding
    assert!(state.fed == 500, "Invalid fed value");


    // feed the tamagotchi
    let res = program.send(OWNER, TmgAction::Feed);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Feed(500 + FILL_PER_FEED));
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // play with tamagotchi
    let res = program.send(OWNER, TmgAction::Play);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Play(500 + FILL_PER_ENTERTAINMENT));
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // rest with tamagotchi
    let res = program.send(OWNER, TmgAction::Sleep);
    let expected_log = Log::builder()
        .dest(OWNER)
        .payload(TmgEvent::Sleep(500 + FILL_PER_SLEEP));
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));


    // read the state
    // assert state is changed
    let state: Tamagotchi = program.read_state().expect("Error while reading the state");
    assert!(state.fed == 500 + FILL_PER_FEED, "Invalid fed value");
    assert!(state.happy == 500 + FILL_PER_ENTERTAINMENT, "Invalid happy value");
    assert!(state.rested == 500 + FILL_PER_SLEEP, "Invalid happy value");
}


