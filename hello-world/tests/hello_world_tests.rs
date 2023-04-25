use gtest::{Log, Program, System};
use hello_world::{TmgAction, TmgEvent};

#[test]
fn hello_test() {
    // initialize env
    let sys = System::new();
    
    // initialize program
    Program::from_file(
        &sys, 
        "./target/wasm32-unknown-unknown/release/hello_world.wasm"
    );
    let program = Program::current(&sys);

    // send tamagotchi name (init)
    let actor_id = 3;
    let mut res = program.send_bytes(actor_id, String::from("Satoshi"));

    let mut expected_log = Log::builder()
        .dest(actor_id)
        .payload(String::from("Success!"));

    assert!(!res.log().is_empty());
    assert!(!res.main_failed());
    assert!(res.contains(&expected_log));

    // test for TmgAction::Name
    res = program.send(actor_id, TmgAction::Name);

    expected_log = Log::builder()
        .dest(actor_id)
        .payload(TmgEvent::Name(String::from("Satoshi")));

    assert!(res.contains(&expected_log));
    assert!(!res.main_failed());
}