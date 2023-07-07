use gtest::{Program, System, Log};
use program_factory::*;

const OWNER: u64 = 100;
const BUYER: u64 = 101;
const PROGRAM_ID: u64 = 1;

fn init_escrow_factory(sys: &System) {
    let escrow_code_id = sys.submit_code("./escrow/target/wasm32-unknown-unknown/debug/escrow.opt.wasm");
    let escrow_factory = Program::current(&sys);
    let res = escrow_factory.send(OWNER, escrow_code_id);

    assert!(!res.main_failed());
    assert!(res.log().is_empty());
}

#[test]
fn create_escrow() {
    // initialize env
    let sys = System::new();

    init_escrow_factory(&sys);
    let program = sys.get_program(PROGRAM_ID);

    // send msg to create escrow
    let price: u128 = 0;
    let mut res = program.send(OWNER, FactoryAction::CreateEscrow{
        seller: OWNER.into(),
        buyer: BUYER.into(),
        price: price 
    });
    assert!(!res.main_failed());
    assert!(!res.log().is_empty());

    // send msg to create escrow
    let price: u128 = 9999;
    res = program.send(OWNER, FactoryAction::CreateEscrow{
        seller: BUYER.into(),
        buyer: OWNER.into(),
        price: price 
    });
    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
}

#[test]
fn deposit_confirm_escrow() {
    // initialize env
    let sys = System::new();

    init_escrow_factory(&sys);
    let program = sys.get_program(PROGRAM_ID);

    // send msg to create escrow
    let price: u128 = 0;
    let _ = program.send(OWNER, FactoryAction::CreateEscrow{
        seller: OWNER.into(),
        buyer: BUYER.into(),
        price: price 
    });

    // send deposit action
    let escrow_id = 1;
    let mut res = program.send(BUYER, FactoryAction::Deposit(escrow_id));
    let mut expected_log = Log::builder()
        .dest(BUYER)
        .payload(FactoryEvent::Deposited(escrow_id));

    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
    assert!(res.contains(&expected_log));

    // send confirm delivery action 
    // in case assets transferred to the buyer
    res = program.send(BUYER, FactoryAction::ConfirmDelivery(escrow_id));
    expected_log = Log::builder()
        .dest(BUYER)
        .payload(FactoryEvent::DeliveryConfirmed(escrow_id));

    assert!(!res.main_failed());
    assert!(!res.log().is_empty());
    assert!(res.contains(&expected_log));
}