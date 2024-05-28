use scrypto::prelude::*;
use scrypto_unit::*;
use escrow::token_quantity::TokenQuantity;
use escrow::AllowanceLifeCycle;
use std::fs::File;
use std::io::Write;
use std::env;

mod common;
mod manifests;
mod mock_dex_manifests;

use common::*;
use manifests::*;
use mock_dex_manifests::*;


/// In this scenario, Alice knows of somewhere she can effectively
/// sell MEME for 0.0015 XRD per and she also knows that MEME price on
/// the markets sometimes dips deep down from its usual price of
/// 0.005; and she wants to exploit this by buying as much cheap MEME
/// as she can get at 0.001 XRD per whenever the price spikes down (so
/// she can later sell it into the 0.0015 market, which is not shown
/// in this scenario). She has 1000 XRD to spend on this. She knows
/// that MEME is traded on four different exchanges and she has the
/// difficult job of deciding how to distribute her 1000 XRD between
/// those DEXes to maximize her MEME purchases. Sudden MEME dips are
/// usually limited to just a single exchange as the price tends to
/// recover fast. She might put 250 XRD on each exchange but she feels
/// like this way she only really gets to utilize 25% of her funds
/// since it's rare for multiple exchanges to see collapses at the
/// same time.
///
/// Lucky for Alice, all these exchanges support Escrow-backed limit
/// buys and so what she does is put her 1000 XRD into an Escrow,
/// issue four Allowances for infinite (effectively max 1000) XRD
/// each, and establish limit buys backed by those allowances on all
/// four exchanges. Now all exchanges can pull on all of her 1000 XRD
/// to fill her up with MEME whenever it spikes down.
///
/// To generate a report displaying Alice's MEME account and escrow
/// holdings for the relevant trading steps in the scenario, set the
/// environment variable SCENARIO2_LOG_FILE to a filename to write to,
/// e.g. on Linux it might look like this:
///
/// ```
/// $ SCENARIO2_LOG_FILE=report.csv scrypto test scenario2
/// ```
///
/// The file will be in csv (comma-separated values) format which can
/// be easily imported into most spreadsheets etc.
#[test]
fn scenario_2() {
    let (mut test_runner, owner, package) = setup_for_test();

    // Create MEME
    let meme_resource = test_runner.create_fungible_resource(dec!(10_000_000_000),
                                                             18,
                                                             owner.account);

    // Create the four DEXes
    let mock_dex_1 = instantiate_mock_dex(&mut test_runner,
                                          &owner,
                                          package,
                                          meme_resource);
    let mock_dex_2 = instantiate_mock_dex(&mut test_runner,
                                          &owner,
                                          package,
                                          meme_resource);
    let mock_dex_3 = instantiate_mock_dex(&mut test_runner,
                                          &owner,
                                          package,
                                          meme_resource);
    let mock_dex_4 = instantiate_mock_dex(&mut test_runner,
                                          &owner,
                                          package,
                                          meme_resource);

    // Define user Alice
    let alice = make_user(&mut test_runner, Some("alice"));
    let alice_escrow = call_instantiate(&mut test_runner, &alice, package);
    let alice_badge_res = test_runner.create_non_fungible_resource(alice.account);
    let alice_badge = NonFungibleGlobalId::new(alice_badge_res, 1.into());

    // Define user Bob
    let bob = make_user(&mut test_runner, Some("bob"));
    give_tokens(
        &mut test_runner,
        &owner.account,
        &owner.nfgid,
        &bob.account,
        &meme_resource,
        TokenQuantity::Fungible(dec!(1_000_000_000)),
    );

    // Define user Charlie
    let charlie = make_user(&mut test_runner, Some("charlie"));
    give_tokens(
        &mut test_runner,
        &owner.account,
        &owner.nfgid,
        &charlie.account,
        &meme_resource,
        TokenQuantity::Fungible(dec!(1_000_000_000)),
    );

    // A convenience structure holding the accounts we want to
    // generate reports for
    let report_accounts = (meme_resource,
                           vec![&alice],
                           vec![("Alice Escrow", &alice_escrow, &alice, &alice_badge)]);
    // If the SCENARIO2_LOG_FILE environment variable is defined we will
    // write a test log to that file.
    let log =
        if let Ok(path) = env::var("SCENARIO2_LOG_FILE") {
            Some(File::create(path).expect("failed to make SCENARIO2_LOG_FILE"))
        } else { None };
    let log_ref = log.as_ref();
    
    write_report_header(log_ref, &report_accounts);
    
    // 1. Alice puts 1000 XRD into Escrow
    call_deposit_funds(
        &mut test_runner,
        &alice,
        alice_escrow,
        &alice_badge,
        None,
        XRD,
        dec!(1_000),
        true,
    );

    // 2. Alice creates four Allowances, each for infinite XRD

    // (there is only 1000 XRD in the escrow so effectively it is
    // limited by this)

    let allowance_for_dex_1 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            XRD,
            None);
    let allowance_for_dex_2 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            XRD,
            None);
    let allowance_for_dex_3 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            XRD,
            None);
    let allowance_for_dex_4 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            XRD,
            None);
    
    // 3. Alice places limit buy orders on each DEX, having them pull
    // funds from the Allowances
    call_limit_buy_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_1,
        &alice_badge,
        dec!(0.001), // 1 XRD buys 1k MEME
        Some(alice_escrow),
        &allowance_for_dex_1,
        true,
    );
    call_limit_buy_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_2,
        &alice_badge,
        dec!(0.001),
        Some(alice_escrow),
        &allowance_for_dex_2,
        true,
    );
    call_limit_buy_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_3,
        &alice_badge,
        dec!(0.001),
        Some(alice_escrow),
        &allowance_for_dex_3,
        true,
    );
    call_limit_buy_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_4,
        &alice_badge,
        dec!(0.001),
        Some(alice_escrow),
        &allowance_for_dex_4,
        true,
    );

    write_report(&mut test_runner, log_ref, "3. Initial market", &report_accounts);

    // 4. Bob sells 100k MEME into DEX 2, simulating a sudden spike
    // down in price
    call_market_sell_direct(
        &mut test_runner,
        &bob,
        mock_dex_2,
        None, // trader
        None, // escrow
        meme_resource,
        dec!(100_000),
        true,
    );

    write_report(&mut test_runner, log_ref, "4. Bob sold", &report_accounts);

    // 5. Charlie sells 1M MEME into DEX 3, simulating another sudden
    // spike down
    call_market_sell_direct(
        &mut test_runner,
        &charlie,
        mock_dex_3,
        None, // trader
        None, // escrow
        meme_resource,
        dec!(1_000_000),
        true,
    );

    write_report(&mut test_runner, log_ref, "5. Charlie sold (final state)", &report_accounts);

    // Bob and Charlie have now caused one downward spike each on
    // different exchanges, with the remaining two exchanges having
    // had no particular market movements. Alice was able to put her
    // entire 1000 XRD into those two, without needing to predict them
    // beforehand which she would have had to do had she not employed
    // an Escrow based trading solution.

    
    // Let's verify that Alice now has the 1M MEME she could afford
    let alice_escrow_meme_balance = 
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        meme_resource);

    assert_eq!(dec!(1_000_000), alice_escrow_meme_balance, "Alice's MEME balance");

    // (Alice now goes on to sell these 1M MEME to her secret @0.0015
    // XRD market, and possibly then uses those proceeds to refinance
    // her Escrow for one more round - not shown here)
}

/// Writes a report header to file, of the type
///
/// Activity,bob,charlie,debbie,eric,Alice Escrow,Fiona Escrow,
fn write_report_header(destination: Option<&File>,
                       accounts: &(ResourceAddress,
                                   Vec<&User>,
                                   Vec<(&str, &ComponentAddress, &User, &NonFungibleGlobalId)>))
{
    if let Some(mut file) = destination {
        write!(file, "Activity,").unwrap();
        for user in &accounts.1 {
            write!(file, "{},", user.display_name()).unwrap();
        }
        for escrow in &accounts.2 {
            write!(file, "{},", escrow.0).unwrap();
        }
        writeln!(file, "").unwrap();
    }
}

/// Writes a report row to file, of the type
///
/// Row Title,1000,0,5000,50,1000000,100000,
fn write_report(test_runner: &mut DefaultTestRunner,
                destination: Option<&File>,
                row_title: &str,
                accounts: &(ResourceAddress,
                            Vec<&User>,
                            Vec<(&str, &ComponentAddress, &User, &NonFungibleGlobalId)>))
{
    if let Some(mut file) = destination {
        write!(file, "{},", row_title).unwrap();
        for user in &accounts.1 {
            write!(file,
                   "{},",
                   test_runner.get_component_balance(user.account, accounts.0)).unwrap();
        }
        for (_, escrow, user, badge) in &accounts.2 {
            let escrow_balance =
                call_read_funds(test_runner,
                                user,
                                **escrow,
                                badge,
                                accounts.0);
            write!(file, "{},", escrow_balance).unwrap();
        }
        writeln!(file, "").unwrap();
    }
}
