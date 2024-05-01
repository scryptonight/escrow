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


/// Alice creates 1 billion MEME tokens and wants to share them with
/// the world (through selling them for cold hard coin, of
/// course). She wants to target four different DEXes for this:
/// DominantDEX, PrettyBigDEX, SmallDEX and NicheDEX (listed in order
/// from biggest to smallest in expected trade volume).
///
/// She considered distributing her MEME across them by
/// 50%/25%/15%/10% respectively but didn't care for having to
/// manually shuffle MEME between them should some see more trade than
/// expected. Therefore she instead puts all her MEME into Escrow, and
/// creates Allowances to withdraw infinite MEME then puts one of each
/// of those Allowances on each DEX.
///
/// This causes all the four DEXes to draw from the Escrow, and all of
/// them will run out at the same time, i.e. when the Escrow is
/// empty. No rebalancing is necessary for Alice and she can just sit
/// back and watch the cash roll in.
///
/// Demonstrating the elegance of Escrow-based distribution, this
/// scenario takes us through a full sellout with Bob, Charlie and
/// Debbie emptying out the markets of all the MEME. Note that Debbie
/// exclusively buys from NicheDEX and she buys a lot, which normally
/// would have caused Alice to have to redistribute some MEME to there
/// from e.g. DominantDEX - but that is previous century thinking for
/// people that didn't have an Escrow component helping them out.
///
/// In this scenario, for clarity, we have a very small number of
/// transactions buying out the entire stock but you can easily
/// imagine a situation where the same result is reached through
/// thousands of transactions from a large number of buyers.
///
/// As an added bonus, after the full sellout we demonstrate that if
/// Alice were to receive some MEME back (in our case, Bob just gives
/// her some) she can simply put that straight into Escrow and since
/// the infinite Allowances are still on the DEXes this refills the
/// markets for someone to come along and buy more.
///
/// To generate a report displaying the accounts of all the users for
/// the trading steps in the scenario, set the environment variable
/// SCENARIO1_LOG_FILE to a filename to write to, e.g. on Linux it
/// might look like this:
///
/// ```
/// $ SCENARIO1_LOG_FILE=report.csv scrypto test scenario1
/// ```
///
/// The file will be in csv (comma-separated values) format which can
/// be easily imported into most spreadsheets etc.
#[test]
fn scenario_1() {
    let (mut test_runner, owner, package) = setup_for_test();

    // Create MEME
    let meme_resource = test_runner.create_fungible_resource(dec!(1_000_000_000),
                                                             18,
                                                             owner.account);

    // Create the four DEXes
    let mock_dex_dominant = instantiate_mock_dex(&mut test_runner,
                                                 &owner,
                                                 package,
                                                 meme_resource);
    let mock_dex_pretty_big = instantiate_mock_dex(&mut test_runner,
                                                   &owner,
                                                   package,
                                                   meme_resource);
    let mock_dex_small = instantiate_mock_dex(&mut test_runner,
                                              &owner,
                                              package,
                                              meme_resource);
    let mock_dex_niche = instantiate_mock_dex(&mut test_runner,
                                              &owner,
                                              package,
                                              meme_resource);

    // Define user Alice
    let alice = make_user(&mut test_runner, Some("alice"));
    let alice_escrow = call_instantiate(&mut test_runner, &alice, package);
    give_tokens(
        &mut test_runner,
        &owner.account,
        &owner.nfgid,
        &alice.account,
        &meme_resource,
        TokenQuantity::Fungible(dec!(1_000_000_000)),
    );
    let alice_badge_res = test_runner.create_non_fungible_resource(alice.account);
    let alice_badge = NonFungibleGlobalId::new(alice_badge_res, 1.into());

    // Define user Bob
    let bob = make_user(&mut test_runner, Some("bob"));

    // Define user Charlie
    let charlie = make_user(&mut test_runner, Some("charlie"));

    // Define user Debbie
    let debbie = make_user(&mut test_runner, Some("debbie"));

    // A convenience structure holding the accounts we want to
    // generate reports for
    let report_accounts = (meme_resource,
                           vec![&bob, &charlie, &debbie],
                           vec![("Alice Escrow", &alice_escrow, &alice, &alice_badge)]);
    // If the SCENARIO1_LOG_FILE environment variable is defined we will
    // write a test log to that file.
    let log =
        if let Ok(path) = env::var("SCENARIO1_LOG_FILE") {
            Some(File::create(path).expect("failed to make SCENARIO1_LOG_FILE"))
        } else { None };
    let log_ref = log.as_ref();
    
    write_report_header(log_ref, &report_accounts);
    
    // 1. Alice puts her MEME into Escrow
    call_deposit_funds(
        &mut test_runner,
        &alice,
        alice_escrow,
        &alice_badge,
        None,
        meme_resource,
        dec!(1_000_000_000),
        true,
    );

    // 2. Alice creates four Allowances, each for infinite MEME
    let allowance_for_dominant_dex =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            meme_resource,
            None);
    let allowance_for_pretty_big_dex =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            meme_resource,
            None);
    let allowance_for_small_dex =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            meme_resource,
            None);
    let allowance_for_niche_dex =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Repeating{min_delay:None},
            meme_resource,
            None);
    
    // 3. Alice places limit sell orders on each DEX, having them pull
    // funds from the Allowances
    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_dominant,
        &alice_badge,
        dec!(0.00001), // 1 XRD buys 100k MEME
        Some(alice_escrow),
        &allowance_for_dominant_dex,
        true,
    );
    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_pretty_big,
        &alice_badge,
        dec!(0.00001),
        Some(alice_escrow),
        &allowance_for_pretty_big_dex,
        true,
    );
    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_small,
        &alice_badge,
        dec!(0.00001),
        Some(alice_escrow),
        &allowance_for_small_dex,
        true,
    );
    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_niche,
        &alice_badge,
        dec!(0.00001),
        Some(alice_escrow),
        &allowance_for_niche_dex,
        true,
    );

    write_report(&mut test_runner, log_ref, "3. Initial market", &report_accounts);

    // 4. Bob buys 5M MEME from DominantDEX
    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_dominant,
        None, // trader
        None, // escrow
        dec!(50),
        true,
    );

    write_report(&mut test_runner, log_ref, "4. Bob bought", &report_accounts);

    // 5. Charlie buys 100M MEME from SmallDEX
    call_market_buy_direct(
        &mut test_runner,
        &charlie,
        mock_dex_small,
        None, // trader
        None, // escrow
        dec!(1000),
        true,
    );

    write_report(&mut test_runner, log_ref, "5. Charlie bought", &report_accounts);

    // 6. Bob buys 200M MEME from PrettyBigDEX
    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_pretty_big,
        None, // trader
        None, // escrow
        dec!(2000),
        true,
    );

    write_report(&mut test_runner, log_ref, "6. Bob bought again", &report_accounts);

    // 7. Debbie buys 695M MEME from NicheDEX, thus clearing out the
    // global market
    call_market_buy_direct(
        &mut test_runner,
        &debbie,
        mock_dex_niche,
        None, // trader
        None, // escrow
        dec!(7000),
        true,
    );

    write_report(&mut test_runner, log_ref, "7. Debbie bought", &report_accounts);

    // 8. Alice somehow receives some MEME back (here Bob just gives
    // her some) - and she promptly puts them into escrow so the
    // markets can access them
    give_tokens(
        &mut test_runner,
        &bob.account,
        &bob.nfgid,
        &alice.account,
        &meme_resource,
        TokenQuantity::Fungible(dec!(1_000_000)), // Bob gives 1M MEME to Alice
    );
    call_deposit_funds(
        &mut test_runner,
        &alice,
        alice_escrow,
        &alice_badge,
        None,
        meme_resource,
        dec!(1_000_000), // Alice puts them into Escrow
        true,
    );
    // Note that Alice didn't explicitly add these MEME to any DEX,
    // the MEME instead just implicitly becomes available there
    // because they're now in the Escrow that the DEXes already have
    // Allowances to.
    
    write_report(&mut test_runner, log_ref, "8. Alice added more to market", &report_accounts);

    // 9. Debbie quickly hoovers up these as well, but from SmallDEX
    // this time because why not
    call_market_buy_direct(
        &mut test_runner,
        &debbie,
        mock_dex_small,
        None, // trader
        None, // escrow
        dec!(1_000),
        true,
    );

    write_report(&mut test_runner, log_ref, "9. Debbie bought again", &report_accounts);

    // Let's verify that all buyers now have their MEME
    let bob_meme_balance = test_runner.get_component_balance(bob.account, meme_resource);
    let charlie_meme_balance = test_runner.get_component_balance(charlie.account, meme_resource);
    let debbie_meme_balance = test_runner.get_component_balance(debbie.account, meme_resource);

    assert_eq!(dec!(204_000_000), bob_meme_balance, "Bob's MEME balance");
    assert_eq!(dec!(100_000_000), charlie_meme_balance, "Charlie's MEME balance");
    assert_eq!(dec!(696_000_000), debbie_meme_balance, "Debbie's MEME balance");
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
