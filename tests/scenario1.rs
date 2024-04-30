use escrow::token_quantity::TokenQuantity;
use escrow::{AllowanceLifeCycle, AllowanceNfData};
use radix_engine::blueprints::resource::NonFungibleVaultError;
use radix_engine::errors::*;
use radix_engine::transaction::{CommitResult, TransactionReceipt};
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

mod common;
mod manifests;

use common::*;
use manifests::*;


/// This tests that the MockDex works are intended, and that it
/// interfaces well with Escrow.
///
/// This test would be more properly put inside its own
/// mock_dex_test.rs test file but it will live here for now.
#[test]
fn scenario_1_basic_tests() {
    let (mut test_runner, owner, package) = setup_for_test();

    let meme_resource = test_runner.create_fungible_resource(dec!(1000000000), 18, owner.account);

    let manifest = ManifestBuilder::new()
        .call_function(
            package,
            "MockDex",
            "instantiate_mock_dex",
            manifest_args!(meme_resource),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&owner.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    let mock_dex_component = receipt.expect_commit_success().new_component_addresses()[0];

    // Define user Alice
    let alice = make_user(&mut test_runner, Some("alice"));
    let alice_escrow = call_instantiate(&mut test_runner, &alice, package);
    give_tokens(
        &mut test_runner,
        &owner.account,
        &owner.nfgid,
        &alice.account,
        &meme_resource,
        TokenQuantity::Fungible(dec!(1000000000)),
    );
    let alice_badge_res = test_runner.create_non_fungible_resource(alice.account);
    let alice_badge = NonFungibleGlobalId::new(alice_badge_res, 1.into());

    // Define user Bob
    let bob = make_user(&mut test_runner, Some("bob"));
    let bob_escrow = call_instantiate(&mut test_runner, &bob, package);
    let bob_badge_res = test_runner.create_non_fungible_resource(bob.account);
    let bob_badge = NonFungibleGlobalId::new(bob_badge_res, 1.into());

    // Alice puts some MEME into escrow
    call_deposit_funds(
        &mut test_runner,
        &alice,
        alice_escrow,
        &alice_badge,
        None,
        meme_resource,
        dec!(1_000_000),
        true,
    );
    // Alice puts some XRD into escrow
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

    let alice_escrow_xrd_pre =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    // Bob puts some XRD into escrow
    call_deposit_funds(
        &mut test_runner,
        &bob,
        bob_escrow,
        &bob_badge,
        None,
        XRD,
        dec!("1000"),
        true,
    );

    // TRADING ON THE SELL BOOK
    
    // Alice puts out several tranches of MEME for sale

    let alice_meme_balance_pre = test_runner.get_component_balance(alice.account, meme_resource);

    // 1000 MEME @ 0.01
    call_limit_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.01),
        Some(alice_escrow),
        meme_resource,
        dec!(1000),
        true,
    );

    // 1000 MEME @ 0.001
    call_limit_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.001),
        Some(alice_escrow),
        meme_resource,
        dec!(1000),
        true,
    );

    // 1000 MEME @ 0.0001
    call_limit_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.0001),
        Some(alice_escrow),
        meme_resource,
        dec!(1000),
        true,
    );

    // 1000 MEME @ 0.1
    call_limit_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.1),
        Some(alice_escrow),
        meme_resource,
        dec!(1000),
        true,
    );

    // 1000 MEME @ 1.0
    call_limit_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(1.0),
        Some(alice_escrow),
        meme_resource,
        dec!(1000),
        true,
    );

    let alice_meme_balance_post = test_runner.get_component_balance(alice.account, meme_resource);

    assert_eq!(
        alice_meme_balance_pre - 5000,
        alice_meme_balance_post,
        "Alice should be 5k MEME down"
    );

    // Bob is hoping to buy some cheap MEME
    let bob_meme_balance_pre = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);

    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        None, // trader
        None, // escrow
        dec!(11),
        true,
    );

    let bob_meme_balance_post = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);

    assert_eq!(
        bob_xrd_balance_pre - 11,
        bob_xrd_balance_post,
        "Bob should be 11 XRD down"
    );
    assert_eq!(
        bob_meme_balance_pre + 2990,
        bob_meme_balance_post,
        "Bob should be 2990 MEME up"
    );


    // TRADING ON THE BUY BOOK

    // Bob puts in limit buy orders
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);
    call_limit_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.01),
        None,
        dec!(1000),
        true,
    );
    call_limit_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.001),
        None,
        dec!(1000),
        true,
    );
    call_limit_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.02),
        None,
        dec!(1000),
        true,
    );

    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);

    assert_eq!(
        bob_xrd_balance_pre - dec!(3000),
        bob_xrd_balance_post,
        "Bob should be 3000 XRD down"
    );
    
    // Alice sells into them

    let alice_xrd_balance_pre = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_pre = test_runner.get_component_balance(alice.account, meme_resource);

    call_market_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        None, // trader
        None, // escrow
        meme_resource,
        dec!(100_000),
        true,
    );
    
    let alice_xrd_balance_post = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_post = test_runner.get_component_balance(alice.account, meme_resource);

    assert_eq!(
        alice_xrd_balance_pre + dec!(1500),
        alice_xrd_balance_post,
        "Alice should be 1500 XRD up"
    );
    assert_eq!(
        alice_meme_balance_pre - dec!(100_000),
        alice_meme_balance_post,
        "Alice should be 100k MEME down"
    );

    // TRADING ON THE SELL BOOK AGAIN
    
    // Bob clears out the rest of the MEME selling order book
    let bob_meme_balance_pre = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);

    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        None, // trader
        None, // escrow
        dec!(2000),
        true,
    );

    let bob_meme_balance_post = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);

    assert_eq!(
        bob_xrd_balance_pre - dec!(1100.1),
        bob_xrd_balance_post,
        "Bob should be 1100.1 XRD down"
    );
    assert_eq!(
        bob_meme_balance_pre + 2010,
        bob_meme_balance_post,
        "Bob should be 2010 MEME up"
    );

    // Buying from an empty market
    let bob_meme_balance_pre = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);

    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        None, // trader
        None, // escrow
        dec!(2000),
        true,
    );

    let bob_meme_balance_post = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);

    assert_eq!(
        bob_xrd_balance_pre, bob_xrd_balance_post,
        "Bob should have spent no XRD"
    );
    assert_eq!(
        bob_meme_balance_pre,
        bob_meme_balance_post,
        "Bob should have received no MEME"
    );

    
    // TRADING ON THE BUY BOOK AGAIN

    // Alice clears out the rest of the buy book

    let alice_xrd_balance_pre = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_pre = test_runner.get_component_balance(alice.account, meme_resource);

    call_market_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        None, // trader
        None, // escrow
        meme_resource,
        dec!(1_200_000),
        true,
    );
    
    let alice_xrd_balance_post = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_post = test_runner.get_component_balance(alice.account, meme_resource);

    assert_eq!(
        alice_xrd_balance_pre + dec!(1500),
        alice_xrd_balance_post,
        "Alice should be 1500 XRD up"
    );
    assert_eq!(
        alice_meme_balance_pre - dec!(1_050_000),
        alice_meme_balance_post,
        "Alice should be 1.05M MEME down"
    );


    // Selling into an empty buy book
    let alice_xrd_balance_pre = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_pre = test_runner.get_component_balance(alice.account, meme_resource);

    call_market_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        None, // trader
        None, // escrow
        meme_resource,
        dec!(1_000_000),
        true,
    );
    
    let alice_xrd_balance_post = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_post = test_runner.get_component_balance(alice.account, meme_resource);

    assert_eq!(
        alice_xrd_balance_pre,
        alice_xrd_balance_post,
        "Alice should have received no XRD"
    );
    assert_eq!(
        alice_meme_balance_pre,
        alice_meme_balance_post,
        "Alice should have spent no MEME"
    );

    // Verify that alice_escrow now has +1111.1 XRD in it from her
    // limit sells that got paid into escrow.
    let alice_escrow_xrd_post =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);
    assert_eq!(alice_escrow_xrd_pre + dec!("1111.1"),
               alice_escrow_xrd_post,
               "Alice should have earnt 1111.1 XRD");


    // Now test Market trade with payout to Escrow

    // Alice puts some more MEME into the sell book

    // 1000 MEME @ 0.1
    call_limit_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.1),
        Some(alice_escrow),
        meme_resource,
        dec!(1000),
        true,
    );

    // Bob market buys 10 MEME into Escrow
    let bob_meme_balance_pre = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);
    let bob_escrow_meme_pre =
        call_read_funds(&mut test_runner,
                        &bob,
                        bob_escrow,
                        &bob_badge,
                        meme_resource);

    // Bob spends 1 XRD on 10 MEME, asking for the MEME to be put into
    // escrow
    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        Some(&bob_badge), // trader
        Some(bob_escrow), // escrow
        dec!(1),
        true,
    );

    let bob_meme_balance_post = test_runner.get_component_balance(bob.account, meme_resource);
    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);
    let bob_escrow_meme_post =
        call_read_funds(&mut test_runner,
                        &bob,
                        bob_escrow,
                        &bob_badge,
                        meme_resource);

    assert_eq!(bob_xrd_balance_pre - 1,
               bob_xrd_balance_post,
               "Bob should have spent 1 XRD");
    assert_eq!(bob_meme_balance_pre,
               bob_meme_balance_post,
               "Bob should have received 0 MEME into his account");
    assert_eq!(bob_escrow_meme_pre + 10,
               bob_escrow_meme_post,
               "Bob should have received 10 MEME into escrow");


    // Bob puts some XRD into the buy book
    call_limit_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.01),
        None,
        dec!(10),
        true,
    );

    // Alice sells MEME into the buy book, collecting in Escrow
    let alice_xrd_balance_pre = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_pre = test_runner.get_component_balance(alice.account, meme_resource);
    let alice_escrow_xrd_pre =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    call_market_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        Some(&alice_badge), // trader
        Some(alice_escrow), // escrow
        meme_resource,
        dec!(1000),
        true,
    );
    
    let alice_xrd_balance_post = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_post = test_runner.get_component_balance(alice.account, meme_resource);
    let alice_escrow_xrd_post =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    assert_eq!(
        alice_xrd_balance_pre,
        alice_xrd_balance_post,
        "Alice should not have received XRD to her account"
    );
    assert_eq!(
        alice_meme_balance_pre - dec!(1000),
        alice_meme_balance_post,
        "Alice should be 1000 MEME down"
    );
    assert_eq!(alice_escrow_xrd_pre + 10,
               alice_escrow_xrd_post,
               "Alice should have received 10 XRD into escrow");


    // Clear out the books, just to clean up

    call_market_buy_direct(
        &mut test_runner,
        &bob,
        mock_dex_component,
        None, // trader
        None, // escrow
        dec!(2000),
        true,
    );
    call_market_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        None, // trader
        None, // escrow
        meme_resource,
        dec!(1_000_000),
        true,
    );
    

    
    // Test limit trade with tokens taken from Escrow

    // Alice makes some allowances and puts them up for trade at
    // different price points.
    let allowance_alice_meme_10k_1 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Accumulating,
            meme_resource,
            Some(TokenQuantity::Fungible(dec!(10_000))));
    let allowance_alice_meme_10k_2 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Accumulating,
            meme_resource,
            Some(TokenQuantity::Fungible(dec!(10_000))));
    let allowance_alice_meme_10k_3 =
        call_mint_allowance(
            &mut test_runner,
            &alice,
            alice_escrow,
            &alice_badge,
            None,
            0,
            AllowanceLifeCycle::Accumulating,
            meme_resource,
            Some(TokenQuantity::Fungible(dec!(10_000))));

    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.1),
        Some(alice_escrow),
        &allowance_alice_meme_10k_1,
        true,
    );
    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(0.01),
        Some(alice_escrow),
        &allowance_alice_meme_10k_2,
        true,
    );
    call_limit_sell_with_escrow(
        &mut test_runner,
        &alice,
        mock_dex_component,
        &alice_badge,
        dec!(1),
        Some(alice_escrow),
        &allowance_alice_meme_10k_3,
        true,
    );


    // Bob buys a few MEME from those
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);
    let bob_meme_balance_pre = test_runner.get_component_balance(bob.account, meme_resource);
    let alice_escrow_meme_pre =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        meme_resource);
    let alice_escrow_xrd_pre =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    let receipt =
        call_market_buy_direct(
            &mut test_runner,
            &bob,
            mock_dex_component,
            None, // trader
            None, // escrow
            dec!(1000),
            true,
        );
    
    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);
    let bob_meme_balance_post = test_runner.get_component_balance(bob.account, meme_resource);
    let alice_escrow_meme_post =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        meme_resource);
    let alice_escrow_xrd_post =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    assert_eq!(bob_xrd_balance_pre - 1000,
               bob_xrd_balance_post,
               "Bob should be down 1k XRD");
    assert_eq!(bob_meme_balance_pre + 19_000,
               bob_meme_balance_post,
               "Bob should be up 19k MEME");
    assert_eq!(alice_escrow_meme_pre - 19_000,
               alice_escrow_meme_post,
               "Alice's escrow should be down 19k MEME");
    assert_eq!(alice_escrow_xrd_pre + 1000,
               alice_escrow_xrd_post,
               "Alice's escrow should be up 1k XRD");



    // Bob buys a few more MEME
    let bob_xrd_balance_pre = test_runner.get_component_balance(bob.account, XRD);
    let bob_meme_balance_pre = test_runner.get_component_balance(bob.account, meme_resource);
    let alice_escrow_meme_pre =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        meme_resource);

    let receipt =
        call_market_buy_direct(
            &mut test_runner,
            &bob,
            mock_dex_component,
            None, // trader
            None, // escrow
            dec!(100),
            true,
        );
    
    let bob_xrd_balance_post = test_runner.get_component_balance(bob.account, XRD);
    let bob_meme_balance_post = test_runner.get_component_balance(bob.account, meme_resource);
    let alice_escrow_meme_post =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        meme_resource);

    assert_eq!(bob_xrd_balance_pre - 100,
               bob_xrd_balance_post,
               "Bob should be down 100 XRD");
    assert_eq!(bob_meme_balance_pre + 1_000,
               bob_meme_balance_post,
               "Bob should be up 1k MEME");
    assert_eq!(alice_escrow_meme_pre - 1_000,
               alice_escrow_meme_post,
               "Alice's escrow should be down 1k MEME");


    // Bob makes some XRD allowances and puts up allowance-based limit
    // buy offers with them

    let allowance_bob_xrd_100_1 =
        call_mint_allowance(
            &mut test_runner,
            &bob,
            bob_escrow,
            &bob_badge,
            None,
            0,
            AllowanceLifeCycle::Accumulating,
            XRD,
            Some(TokenQuantity::Fungible(dec!(100))));
    let allowance_bob_xrd_100_2 =
        call_mint_allowance(
            &mut test_runner,
            &bob,
            bob_escrow,
            &bob_badge,
            None,
            0,
            AllowanceLifeCycle::Accumulating,
            XRD,
            Some(TokenQuantity::Fungible(dec!(100))));
    let allowance_bob_xrd_100_3 =
        call_mint_allowance(
            &mut test_runner,
            &bob,
            bob_escrow,
            &bob_badge,
            None,
            0,
            AllowanceLifeCycle::Accumulating,
            XRD,
            Some(TokenQuantity::Fungible(dec!(100))));
    
    call_limit_buy_with_escrow(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.01),
        Some(bob_escrow),
        &allowance_bob_xrd_100_1,
        true,
    );
    call_limit_buy_with_escrow(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.001),
        Some(bob_escrow),
        &allowance_bob_xrd_100_2,
        true,
    );
    call_limit_buy_with_escrow(
        &mut test_runner,
        &bob,
        mock_dex_component,
        &bob_badge,
        dec!(0.1),
        Some(bob_escrow),
        &allowance_bob_xrd_100_3,
        true,
    );


    // Alice now sells into Bob's limit buys
    let alice_xrd_balance_pre = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_pre = test_runner.get_component_balance(alice.account, meme_resource);
    let alice_escrow_xrd_pre =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    call_market_sell_direct(
        &mut test_runner,
        &alice,
        mock_dex_component,
        Some(&alice_badge), // trader
        Some(alice_escrow), // escrow
        meme_resource,
        dec!(5_000),
        true,
    );
    

    let alice_xrd_balance_post = test_runner.get_component_balance(alice.account, XRD);
    let alice_meme_balance_post = test_runner.get_component_balance(alice.account, meme_resource);
    let alice_escrow_xrd_post =
        call_read_funds(&mut test_runner,
                        &alice,
                        alice_escrow,
                        &alice_badge,
                        XRD);

    assert_eq!(
        alice_xrd_balance_pre,
        alice_xrd_balance_post,
        "Alice should not have received XRD to her account"
    );
    assert_eq!(
        alice_meme_balance_pre - dec!(5000),
        alice_meme_balance_post,
        "Alice should be 5000 MEME down"
    );
    assert_eq!(alice_escrow_xrd_pre + 140,
               alice_escrow_xrd_post,
               "Alice should have received 140 XRD into escrow");
}

fn call_limit_sell_direct(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    mock_dex: ComponentAddress,
    trader: &NonFungibleGlobalId,
    price_in_xrd: Decimal,
    escrow_payout_component: Option<ComponentAddress>,
    meme_resource: ResourceAddress,
    meme_amount_to_sell: Decimal,
    expect_success: bool,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            trader.resource_address(),
            BTreeSet::from([trader.local_id().clone()]),
        )
        .pop_from_auth_zone("trader_proof")
        .withdraw_from_account(user.account, meme_resource, meme_amount_to_sell)
        .take_from_worktop(meme_resource, meme_amount_to_sell, "meme_bucket")
        .call_method_with_name_lookup(mock_dex, "limit_sell_direct", |lookup| {
            manifest_args!(
                lookup.proof("trader_proof"),
                price_in_xrd,
                escrow_payout_component,
                lookup.bucket("meme_bucket")
            )
        })
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

fn call_limit_buy_direct(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    mock_dex: ComponentAddress,
    trader: &NonFungibleGlobalId,
    price_in_xrd: Decimal,
    escrow_payout_component: Option<ComponentAddress>,
    xrd_amount_to_spend: Decimal,
    expect_success: bool,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            trader.resource_address(),
            BTreeSet::from([trader.local_id().clone()]),
        )
        .pop_from_auth_zone("trader_proof")
        .withdraw_from_account(user.account, XRD, xrd_amount_to_spend)
        .take_from_worktop(XRD, xrd_amount_to_spend, "xrd_bucket")
        .call_method_with_name_lookup(mock_dex, "limit_buy_direct", |lookup| {
            manifest_args!(
                lookup.proof("trader_proof"),
                price_in_xrd,
                escrow_payout_component,
                lookup.bucket("xrd_bucket")
            )
        })
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

fn call_market_buy_direct(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    mock_dex: ComponentAddress,
    trader: Option<&NonFungibleGlobalId>,
    escrow_payout_component: Option<ComponentAddress>,
    xrd_to_pay: Decimal,
    expect_success: bool,
) -> TransactionReceipt {
    let mut builder = ManifestBuilder::new();
    if let Some(trader) = &trader {
        builder = builder
            .create_proof_from_account_of_non_fungibles(
                user.account,
                trader.resource_address(),
                BTreeSet::from([trader.local_id().clone()]),
            )
            .pop_from_auth_zone("trader_proof")
    }
    let manifest = builder
        .withdraw_from_account(user.account, XRD, xrd_to_pay)
        .take_from_worktop(XRD, xrd_to_pay, "xrd_bucket")
        .call_method_with_name_lookup(mock_dex, "market_buy_direct", |lookup| {
            manifest_args!(
                if trader.is_some() {
                    Some(lookup.proof("trader_proof"))
                } else {
                    None
                },
                escrow_payout_component,
                lookup.bucket("xrd_bucket")
            )
        })
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

fn call_market_sell_direct(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    mock_dex: ComponentAddress,
    trader: Option<&NonFungibleGlobalId>,
    escrow_payout_component: Option<ComponentAddress>,
    meme_resource: ResourceAddress,
    meme_to_spend: Decimal,
    expect_success: bool,
) -> TransactionReceipt {
    let mut builder = ManifestBuilder::new();
    if let Some(trader) = &trader {
        builder = builder
            .create_proof_from_account_of_non_fungibles(
                user.account,
                trader.resource_address(),
                BTreeSet::from([trader.local_id().clone()]),
            )
            .pop_from_auth_zone("trader_proof")
    }
    let manifest = builder
        .withdraw_from_account(user.account, meme_resource, meme_to_spend)
        .take_from_worktop(meme_resource, meme_to_spend, "meme_bucket")
        .call_method_with_name_lookup(mock_dex, "market_sell_direct", |lookup| {
            manifest_args!(
                if trader.is_some() {
                    Some(lookup.proof("trader_proof"))
                } else {
                    None
                },
                escrow_payout_component,
                lookup.bucket("meme_bucket")
            )
        })
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}



fn call_limit_buy_with_escrow(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    mock_dex: ComponentAddress,
    trader: &NonFungibleGlobalId,
    price_in_xrd: Decimal,
    escrow_payout_component: Option<ComponentAddress>,
    allowance: &NonFungibleGlobalId,
    expect_success: bool,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            trader.resource_address(),
            BTreeSet::from([trader.local_id().clone()]),
        )
        .pop_from_auth_zone("trader_proof")
        .withdraw_non_fungibles_from_account(user.account,
                                             allowance.resource_address(),
                                             BTreeSet::from([allowance.local_id().clone()]))
        .take_all_from_worktop(allowance.resource_address(), "allowance_bucket")
        .call_method_with_name_lookup(mock_dex, "limit_buy_with_escrow", |lookup| {
            manifest_args!(
                lookup.proof("trader_proof"),
                price_in_xrd,
                escrow_payout_component,
                lookup.bucket("allowance_bucket")
            )
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

fn call_limit_sell_with_escrow(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    mock_dex: ComponentAddress,
    trader: &NonFungibleGlobalId,
    price_in_xrd: Decimal,
    escrow_payout_component: Option<ComponentAddress>,
    allowance: &NonFungibleGlobalId,
    expect_success: bool,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            trader.resource_address(),
            BTreeSet::from([trader.local_id().clone()]),
        )
        .pop_from_auth_zone("trader_proof")
        .withdraw_non_fungibles_from_account(user.account,
                                             allowance.resource_address(),
                                             BTreeSet::from([allowance.local_id().clone()]))
        .take_all_from_worktop(allowance.resource_address(), "allowance_bucket")
        .call_method_with_name_lookup(mock_dex, "limit_sell_with_escrow", |lookup| {
            manifest_args!(
                lookup.proof("trader_proof"),
                price_in_xrd,
                escrow_payout_component,
                lookup.bucket("allowance_bucket")
            )
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}
