use escrow_new_io::{EscrowAction, EscrowEvent, InitEscrow};
use gstd::prelude::*;
use gtest::{Log, Program, System};

const BUYER: u64 = 100;
const SELLER: u64 = 101;
const PRICE: u128 = 100_000;
const ESCROW_ID: u64 = 1;
const FACTORY_ID: u64 = 10;

fn init_escrow(sys: &System) {
    sys.init_logger();
    let escrow = Program::current(sys);
    let res = escrow.send(
        FACTORY_ID,
        InitEscrow {
            seller: SELLER.into(),
            buyer: BUYER.into(),
            price: PRICE,
        },
    );
    let log = Log::builder()
        .source(ESCROW_ID)
        .dest(FACTORY_ID)
        .payload(EscrowEvent::ProgramInitialized);
    assert!(res.contains(&log));
}

#[test]
fn deposit() {
    let sys = System::new();
    init_escrow(&sys);

    let escrow = sys.get_program(ESCROW_ID);

    sys.mint_to(FACTORY_ID, PRICE);

    let res = escrow.send_with_value(FACTORY_ID, EscrowAction::Deposit(BUYER.into()), PRICE);
    let log = Log::builder()
        .dest(FACTORY_ID)
        .payload(EscrowEvent::FundsDeposited);
    assert!(res.contains(&log));

    let escrow_balance = sys.balance_of(ESCROW_ID);
    assert_eq!(escrow_balance, PRICE);
}

#[test]
fn deposit_failures() {
    let sys = System::new();
    init_escrow(&sys);

    let escrow = sys.get_program(ESCROW_ID);

    sys.mint_to(FACTORY_ID, 2 * PRICE);
    // must fail since BUYER attaches not enough value
    let res = escrow.send_with_value(
        FACTORY_ID,
        EscrowAction::Deposit(BUYER.into()),
        2 * PRICE - 500,
    );
    assert!(res.main_failed());

    // must fail since the message sender is not BUYER
    let res = escrow.send(FACTORY_ID, EscrowAction::Deposit(SELLER.into()));
    assert!(res.main_failed());

    // successful deposit
    let res = escrow.send_with_value(FACTORY_ID, EscrowAction::Deposit(BUYER.into()), PRICE);
    assert!(!res.main_failed());

    // must fail since the state must be `AwaitingPayment`
    let res = escrow.send_with_value(FACTORY_ID, EscrowAction::Deposit(BUYER.into()), PRICE);
    assert!(res.main_failed());
}
