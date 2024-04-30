use scrypto::prelude::*;
use radix_engine::errors::*;
use transaction::builder::ManifestBuilder;
use escrow::token_quantity::TokenQuantity;
use radix_engine::blueprints::resource::NonFungibleVaultError;
use escrow::{AllowanceLifeCycle, AllowanceNfData};

mod common;
mod manifests;

use common::*;
use manifests::*;

#[test]
fn test_instantiate() {
    let (mut test_runner, owner, package) = setup_for_test();

    let manifest = ManifestBuilder::new()
        .call_function(package,
                       "Escrow",
                       "instantiate_escrow",
                       manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&owner.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }
}



#[test]
fn test_deposit_funds() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    // The first time, our pool entry is created
    let receipt =
        call_deposit_funds(&mut test_runner,
                           &owner,
                           escrow,
                           &owner_badge,
                           None,
                           XRD,
                           dec!("100"),
                           true);
    let result = receipt.expect_commit_success();
    assert_eq!(dec!("100"),
               balance_change_amount(result, test_runner.get_component_vaults(escrow, XRD), XRD),
               "Component should be up 100 XRD");
    drop(receipt);


    // The second time, our existing pool is reused
    let receipt =
        call_deposit_funds(&mut test_runner,
                           &owner,
                           escrow,
                           &owner_badge,
                           None,
                           XRD,
                           dec!("50"),
                           true);
    let result = receipt.expect_commit_success();
    assert_eq!(dec!("50"),
               balance_change_amount(result, test_runner.get_component_vaults(escrow, XRD), XRD),
               "Component should be up 50 XRD");
    drop(receipt);

    assert_eq!(dec!("150"),
               call_read_funds(&mut test_runner,
                               &owner,
                               escrow,
                               &owner_badge,
                               XRD),
               "Owner should now have 150 XRD pooled");
}


#[test]
fn test_withdraw() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    call_deposit_funds(&mut test_runner,
                       &owner,
                       escrow,
                       &owner_badge,
                       None,
                       XRD,
                       dec!("100"),
                       true);

    let result =
        call_withdraw(&mut test_runner,
                      &owner,
                      escrow,
                      &owner_badge,
                      XRD,
                      TokenQuantity::Fungible(dec!("10")));

    assert_eq!(dec!("-10"),
               balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD),
               "Escrow should be down 10 XRD");
    assert_eq!(dec!("10"),
               balance_change_amount(&result, test_runner.get_component_vaults(owner.account, XRD), XRD),
               "User should be up 10 XRD");
}


#[test]
fn test_withdraw_non_fungibles() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let nfts_res =
        create_nft_resource(&mut test_runner,
                            &owner,
                            0,
                            1000,
                            None);

    for _ in 0..20 {
        call_deposit_funds(&mut test_runner,
                           &owner,
                           escrow,
                           &owner_badge,
                           None,
                           nfts_res,
                           dec!("50"),
                           true);
    }

    // Verify withdraw with only named nflids
    let result =
        call_withdraw(&mut test_runner,
                      &owner,
                      escrow,
                      &owner_badge,
                      nfts_res,
                      TokenQuantity::NonFungible(
                          Some([1.into(), 3.into()].into()),
                          None));

    assert_eq!(dec!("-2"),
               balance_change_amount(&result,
                                     test_runner.get_component_vaults(escrow, nfts_res),
                                     nfts_res),
               "Escrow should be down 2 NFTs");
    assert_eq!(dec!("2"),
               balance_change_amount(&result,
                                     test_runner.get_component_vaults(owner.account, nfts_res),
                                     nfts_res),
               "User should be up 2 NFTs");
    drop(result);


    // Verify withdraw with named nflids and amount
    let result =
        call_withdraw(&mut test_runner,
                      &owner,
                      escrow,
                      &owner_badge,
                      nfts_res,
                      TokenQuantity::NonFungible(
                          Some([10.into(), 13.into()].into()),
                          Some(10)));

    assert_eq!(dec!("-12"),
               balance_change_amount(&result,
                                     test_runner.get_component_vaults(escrow, nfts_res),
                                     nfts_res),
               "Escrow should be down 12 NFTs");
    assert_eq!(dec!("12"),
               balance_change_amount(&result,
                                     test_runner.get_component_vaults(owner.account, nfts_res),
                                     nfts_res),
               "User should be up 12 NFTs");
    assert!(get_component_nflids(&mut test_runner, owner.account, nfts_res)
            .is_superset(&[10.into(), 13.into()].into()),
            "User should have the named nflids");

    
    // Verify withdraw with only amount
    let result =
        call_withdraw(&mut test_runner,
                      &owner,
                      escrow,
                      &owner_badge,
                      nfts_res,
                      TokenQuantity::NonFungible(
                          None,
                          Some(10)));

    assert_eq!(dec!("-10"),
               balance_change_amount(&result,
                                     test_runner.get_component_vaults(escrow, nfts_res),
                                     nfts_res),
               "Escrow should be down 10 NFTs");
    assert_eq!(dec!("10"),
               balance_change_amount(&result,
                                     test_runner.get_component_vaults(owner.account, nfts_res),
                                     nfts_res),
               "User should be up 10 NFTs");
}

#[test]
fn test_withdraw_all_of() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    
    // First test fungibles
    
    call_deposit_funds(&mut test_runner,
                       &owner,
                       escrow,
                       &owner_badge,
                       None,
                       XRD,
                       dec!("100"),
                       true);

    let result =
        call_withdraw_all_of(&mut test_runner,
                             &owner,
                             escrow,
                             &owner_badge,
                             XRD);

    assert_eq!(dec!("-100"),
               balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD),
               "Escrow should be down 100 XRD");
    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(owner.account, XRD), XRD),
               "User should be up 100 XRD");

    
    // Then test non-fungibles
    
    let nfts_res =
        test_runner.create_non_fungible_resource(owner.account);

    call_deposit_funds(&mut test_runner,
                       &owner,
                       escrow,
                       &owner_badge,
                       None,
                       nfts_res,
                       dec!("3"),
                       true);

    let result =
        call_withdraw_all_of(&mut test_runner,
                             &owner,
                             escrow,
                             &owner_badge,
                             nfts_res);

    assert_eq!(dec!("-3"),
               balance_change_amount(&result, test_runner.get_component_vaults(escrow, nfts_res), nfts_res),
               "Escrow should be down 3 NFTs");
    assert_eq!(dec!("3"),
               balance_change_amount(&result, test_runner.get_component_vaults(owner.account, nfts_res), nfts_res),
               "User should be up 3 NFTs");
}

#[test]
fn test_subsidize() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let play_resource =
        test_runner.create_fungible_resource(dec!("1000"), 18, owner.account);
    
    call_deposit_funds(&mut test_runner,
                       &owner,
                       escrow,
                       &owner_badge,
                       None,
                       XRD,
                       dec!("100"),
                       true);
    
    let result =
        call_subsidize_and_play(&mut test_runner,
                                &owner,
                                escrow,
                                &owner_badge,
                                dec!("10"),
                                play_resource);

    assert!(balance_change_amount(
        &result,
        test_runner.get_component_vaults(owner.account, XRD),
        XRD).is_zero(),
            "Owner should have not paid the XRD fee");

    assert!(balance_change_amount(
        &result,
        test_runner.get_component_vaults(escrow, XRD),
        XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");
}


#[test]
fn test_subsidize_contingent() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    // The amount of play resource needs to be less than the
    // "ridiculously large" amount that causes a fail in
    // call_subsidize_and_play_impl.
    let play_resource =
        test_runner.create_fungible_resource(dec!("1000"), 18, owner.account);
    
    call_deposit_funds(&mut test_runner,
                       &owner,
                       escrow,
                       &owner_badge,
                       None,
                       XRD,
                       dec!("100"),
                       true);


    // Test contingent fee on a successful tx manifest
    
    let result =
        call_subsidize_contingent_and_play(&mut test_runner,
                                           &owner,
                                           escrow,
                                           &owner_badge,
                                           dec!("10"),
                                           play_resource);

    assert!(balance_change_amount(&result, test_runner.get_component_vaults(owner.account, XRD), XRD).is_zero(),
            "Owner should have not paid the XRD fee");
    assert!(balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");

    
    // Test contingent fee on an unsuccessful tx manifest

    let result =
        call_subsidize_contingent_and_fail(&mut test_runner,
                                           &owner,
                                           escrow,
                                           &owner_badge,
                                           dec!("10"),
                                           play_resource);

    assert!(balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD).is_zero(),
            "Escrow should have not paid the XRD fee");
    assert!(balance_change_amount(&result, test_runner.get_component_vaults(owner.account, XRD), XRD)
            != Decimal::ZERO,
            "Owner should have paid the XRD fee");
}


#[test]
fn test_mint_allowance() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let allowance_nfgid = call_mint_allowance(&mut test_runner,
                                              &owner,
                                              escrow,
                                              &owner_badge,
                                              Some(50),
                                              2,
                                              AllowanceLifeCycle::Accumulating,
                                              XRD,
                                              Some(TokenQuantity::Fungible(dec!("100"))));

    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_nfgid.resource_address(),
        allowance_nfgid.local_id().clone());

    assert_eq!(escrow, nfdata.escrow_pool.0,
               "allowance should reference correct escrow instance");
    assert_eq!(owner_badge, nfdata.escrow_pool.1,
               "owner should own the allowance's pool");
    assert_eq!(Some(50), nfdata.valid_until,
               "valid_until should be as we set it");
    assert_eq!(2, nfdata.valid_from,
               "valid_from should be as we set it");
    assert!(matches!(nfdata.life_cycle, AllowanceLifeCycle::Accumulating),
            "life_cycle should be as we set it");
    assert_eq!(XRD, nfdata.for_resource,
               "for_resource should be as we set it");
    assert_eq!(Some(dec!("100")), nfdata.max_amount.map(|v|v.to_amount()),
               "max_amount should be as we set it");
}


#[test]
fn test_reduce_allowance_to_amount() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let allowance_f_nfgid = call_mint_allowance(&mut test_runner,
                                                &owner,
                                                escrow,
                                                &owner_badge,
                                                Some(50),
                                                2,
                                                AllowanceLifeCycle::Accumulating,
                                                XRD,
                                                Some(TokenQuantity::Fungible(dec!("100"))));

    // Verify negative new_max fails
    let receipt =
        call_reduce_allowance_to_amount(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_f_nfgid.clone(),
                                        dec!("-2"),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2003 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    // Verify successful reduction of Fungible allowance
    call_reduce_allowance_to_amount(&mut test_runner,
                                    &owner,
                                    escrow,
                                    allowance_f_nfgid.clone(),
                                    dec!("25"),
                                    true);

    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_f_nfgid.resource_address(),
        allowance_f_nfgid.local_id().clone());

    assert_eq!(Some(dec!("25")), nfdata.max_amount.map(|v|v.to_amount()),
               "max_amount should have been reduced");


    // Verify too high allowance reduction on Fungible allowance fails
    let receipt =
        call_reduce_allowance_to_amount(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_f_nfgid.clone(),
                                        dec!("200"),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2000 ")
        } else {
            false
        }
    });
    drop(receipt);


    let nf_resaddr =
        create_nft_resource(&mut test_runner,
                            &owner,
                            0,
                            200,
                            None);

    let allowance_nf1_nfgid = call_mint_allowance(&mut test_runner,
                                                 &owner,
                                                 escrow,
                                                 &owner_badge,
                                                 Some(50),
                                                 2,
                                                 AllowanceLifeCycle::Accumulating,
                                                 nf_resaddr,
                                                 Some(TokenQuantity::NonFungible(
                                                     Some((0..50).map(|v|v.into()).collect()),
                                                     Some(100))));

    // Verify decimal number on NonFungible allowance fails
    let receipt =
        call_reduce_allowance_to_amount(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_nf1_nfgid.clone(),
                                        dec!("1.5"),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2004 ")
        } else {
            false
        }
    });
    drop(receipt);

    // Verify successful reduction of NonFungible allowance
    call_reduce_allowance_to_amount(&mut test_runner,
                                    &owner,
                                    escrow,
                                    allowance_nf1_nfgid.clone(),
                                    dec!("25"),
                                    true);

    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_nf1_nfgid.resource_address(),
        allowance_nf1_nfgid.local_id().clone());

    if let Some(TokenQuantity::NonFungible(Some(nflids), Some(amount))) = nfdata.max_amount {
        assert_eq!(25, amount,
                   "max_amount should have been reduced");
        assert_eq!(50, nflids.len(),
                   "nflids quantity should be unchanged");
    } else { panic!("max_amount should be NonFungible"); }


    // Verify too high allowance reduction on NonFungible allowance fails
    let receipt =
        call_reduce_allowance_to_amount(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_nf1_nfgid.clone(),
                                        dec!("50"),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2001 ")
        } else {
            false
        }
    });
    drop(receipt);


    // Make a NonFungible allowance with None max_amount
    let allowance_nf2_nfgid = call_mint_allowance(&mut test_runner,
                                                 &owner,
                                                 escrow,
                                                 &owner_badge,
                                                 Some(50),
                                                 2,
                                                 AllowanceLifeCycle::Accumulating,
                                                 nf_resaddr,
                                                 Some(TokenQuantity::NonFungible(
                                                     Some((0..50).map(|v|v.into()).collect()),
                                                     None)));

    // Verify that allowance reduction on NonFungible allowance with
    // None max_amount fails
    let receipt =
        call_reduce_allowance_to_amount(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_nf2_nfgid.clone(),
                                        dec!("1"),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2002 ")
        } else {
            false
        }
    });
    drop(receipt);
}



#[test]
fn test_reduce_allowance_by_nflids() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let allowance_f1_nfgid = call_mint_allowance(&mut test_runner,
                                                &owner,
                                                escrow,
                                                &owner_badge,
                                                Some(50),
                                                2,
                                                AllowanceLifeCycle::Accumulating,
                                                XRD,
                                                Some(TokenQuantity::Fungible(dec!("100"))));

    // Verify call on Fungible allowance fails
    let receipt =
        call_reduce_allowance_by_nflids(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_f1_nfgid.clone(),
                                        (0..10).map(|v|v.into()).collect(),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2005 ")
        } else {
            false
        }
    });
    drop(receipt);


    let allowance_f2_nfgid = call_mint_allowance(&mut test_runner,
                                                &owner,
                                                escrow,
                                                &owner_badge,
                                                Some(50),
                                                2,
                                                AllowanceLifeCycle::Accumulating,
                                                XRD,
                                                None);

    // Verify call on None allowance fails
    let receipt =
        call_reduce_allowance_by_nflids(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_f2_nfgid.clone(),
                                        (0..10).map(|v|v.into()).collect(),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2007 ")
        } else {
            false
        }
    });
    drop(receipt);


    let nf_resaddr =
        create_nft_resource(&mut test_runner,
                            &owner,
                            0,
                            200,
                            None);

    let allowance_nf3_nfgid = call_mint_allowance(&mut test_runner,
                                                 &owner,
                                                 escrow,
                                                 &owner_badge,
                                                 Some(50),
                                                 2,
                                                 AllowanceLifeCycle::Accumulating,
                                                 nf_resaddr,
                                                 Some(TokenQuantity::NonFungible(
                                                     None,
                                                     Some(100))));

    // Verify call on allowance with None nflids fails
    let receipt =
        call_reduce_allowance_by_nflids(&mut test_runner,
                                        &owner,
                                        escrow,
                                        allowance_nf3_nfgid.clone(),
                                        (0..10).map(|v|v.into()).collect(),
                                        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2006 ")
        } else {
            false
        }
    });
    drop(receipt);


    let allowance_nf4_nfgid = call_mint_allowance(&mut test_runner,
                                                 &owner,
                                                 escrow,
                                                 &owner_badge,
                                                 Some(50),
                                                 2,
                                                 AllowanceLifeCycle::Accumulating,
                                                 nf_resaddr,
                                                 Some(TokenQuantity::NonFungible(
                                                     Some((0..50).map(|v|v.into()).collect()),
                                                     Some(100))));

    // Verify successful reduction of NonFungible allowance
    call_reduce_allowance_by_nflids(&mut test_runner,
                                    &owner,
                                    escrow,
                                    allowance_nf4_nfgid.clone(),
                                    (0..10).map(|v|v.into()).collect(),
                                    true);

    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_nf4_nfgid.resource_address(),
        allowance_nf4_nfgid.local_id().clone());

    if let Some(TokenQuantity::NonFungible(Some(nflids), Some(amount))) = nfdata.max_amount {
        assert_eq!(100, amount,
                   "max_amount should be unchanged");
        assert_eq!(40, nflids.len(),
                   "nflids quantity should be down by 10");
        assert!(nflids == (10..50).map(|v|v.into()).collect::<IndexSet<_>>(),
                "nflids should be 10-49 inclusive");
    } else { panic!("max_amount should be NonFungible"); }
}


#[test]
fn test_withdraw_with_allowance_within_validity_period() {
    let (mut test_runner, alice, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &alice, package);

    let badge_res =
        test_runner.create_non_fungible_resource(alice.account);
    let alice_pool_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let play_resource =
        test_runner.create_fungible_resource(dec!("100000"), 18, alice.account);
    
    call_deposit_funds(&mut test_runner,
                       &alice,
                       escrow,
                       &alice_pool_badge,
                       None,
                       play_resource,
                       dec!("10000"),
                       true);

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::OneOff,
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   None,
                                                   2,
                                                   AllowanceLifeCycle::Repeating{
                                                       min_delay: Some(500)},
                                                   play_resource,
                                                   None);

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_acc_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep2_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    let allowance_resaddr = allowance_1off_nfgid.resource_address();

    let bob = make_user(&mut test_runner, Some("bob"));

    // Bob gets all the allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(4)));


    // Test allowances within their valid period

    set_test_runner_clock(&mut test_runner, 400);

    // Try to withdraw too much
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::Fungible(dec!("10000")),
        false);
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::Fungible(dec!("10000")),
        false);
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::Fungible(dec!("10000")),
        false);


    // Test withdrawal with one-off allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::Fungible(dec!("55")),
        true);
    let result = receipt.expect_commit_ignore_outcome();

    assert_eq!(dec!("55"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 55 funds up");
    assert_eq!(dec!("-1"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, allowance_resaddr),
                                     allowance_resaddr),
               "Bob's one-off allowance should be burnt");


    // Test withdrawal with accumulating allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::Fungible(dec!("40")),
        true);
    let result = receipt.expect_commit_ignore_outcome();

    assert_eq!(dec!("40"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 40 funds up");
    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_resaddr,
        allowance_acc_nfgid.local_id().clone());
    assert_eq!(dec!("60"),
               nfdata.max_amount.unwrap().to_amount(),
               "Accumulating allowance should be down 40 tokens");

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::Fungible(dec!("60")),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("60"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 60 funds up");
    assert_eq!(dec!("-1"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, allowance_resaddr),
                                     allowance_resaddr),
               "Bob's accumulating allowance should be burnt");


    // Test withdrawal with repeating allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::Fungible(dec!("10")),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("10"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 10 funds up");

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::Fungible(dec!("100")),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("100"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 100 funds up");

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::Fungible(dec!("100")),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("100"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 100 funds up");


    // Test withdrawal without max_amount
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        TokenQuantity::Fungible(dec!("1000")),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("1000"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 1000 funds up");

    // Test that we cannot withdraw again (because of min_delay)
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        TokenQuantity::Fungible(dec!("100")),
        false);

    // Set a time that is past the min_delay
    set_test_runner_clock(&mut test_runner, 900);

    // Test that we can withdraw again after the min_delay is over
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        TokenQuantity::Fungible(dec!("100")),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("100"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 100 funds up");
}


#[test]
fn test_withdraw_with_allowance_fails_outside_vailidity_period() {
    let (mut test_runner, alice, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &alice, package);

    let badge_res =
        test_runner.create_non_fungible_resource(alice.account);
    let alice_pool_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let play_resource =
        test_runner.create_fungible_resource(dec!("100000"), 18, alice.account);

    call_deposit_funds(&mut test_runner,
                       &alice,
                       escrow,
                       &alice_pool_badge,
                       None,
                       play_resource,
                       dec!("10000"),
                       true);

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::OneOff,
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   None,
                                                   2,
                                                   AllowanceLifeCycle::Repeating{
                                                       min_delay: Some(500)},
                                                   play_resource,
                                                   None);

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_acc_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep2_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    let allowance_resaddr = allowance_1off_nfgid.resource_address();

    let bob = make_user(&mut test_runner, Some(&"Bob".to_owned()));

    // Bob gets all the allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(4)));


    // Test that allowances fail before they are valid

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::Fungible(dec!("1")),
        false);
        
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::Fungible(dec!("1")),
        false);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::Fungible(dec!("1")),
        false);


    // Test that allowances fail after they are invalid

    set_test_runner_clock(&mut test_runner, 600);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::Fungible(dec!("1")),
        false);
        
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::Fungible(dec!("1")),
        false);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::Fungible(dec!("1")),
        false);
}


#[test]
fn test_withdraw_non_fungibles_with_allowance_within_validity_period() {
    let (mut test_runner, alice, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &alice, package);

    let badge_res =
        test_runner.create_non_fungible_resource(alice.account);
    let alice_pool_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let play_resource =
        create_nft_resource(&mut test_runner,
                            &alice,
                            1,
                            1000,
                            None);

    // A small loop to stay within cost unit limits
    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
    }

    // These allowances use Fungible allowance spec to pull
    // non-fungible resources.

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   Some(500),
                                                   2,
                                                   AllowanceLifeCycle::OneOff,
                                                   play_resource,
                                                   Some(TokenQuantity::Fungible(dec!("10"))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("10"))));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("10"))));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   None,
                                                   2,
                                                   AllowanceLifeCycle::Repeating{
                                                       min_delay: Some(500)},
                                                   play_resource,
                                                   None);

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_acc_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep2_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    let allowance_resaddr = allowance_1off_nfgid.resource_address();

    let bob = make_user(&mut test_runner, Some(&"Bob".to_owned()));

    // Bob gets all the allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(4)));

    // Test allowances within their valid period

    set_test_runner_clock(&mut test_runner, 400);

    // Try to withdraw too much
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(1..20)),
            None),
        false);
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(1..20)),
            None),
        false);
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(1..20)),
            None),
        false);


    // Test withdrawal with one-off allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(1..6)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();

    assert_eq!(dec!("5"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 5 NFTs up");
    assert_eq!(dec!("-1"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, allowance_resaddr),
                                     allowance_resaddr),
               "Bob's one-off allowance should be burnt");


    // Test withdrawal with accumulating allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(11..15)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();

    assert_eq!(dec!("4"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 4 NFTs up");
    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_resaddr,
        allowance_acc_nfgid.local_id().clone());
    assert_eq!(dec!("6"),
               nfdata.max_amount.unwrap().to_amount(),
               "Accumulating allowance should be down 4 NFTs");

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(15..21)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("6"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 6 NFTs up");
    assert_eq!(dec!("-1"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, allowance_resaddr),
                                     allowance_resaddr),
               "Bob's accumulating allowance should be burnt");


    // Test withdrawal with repeating allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(801..805)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("4"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 4 NFTs up");

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(811..821).into()),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("10"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 10 NFTs up");

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(821..831)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("10"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 10 NFTs up");


    // Test withdrawal without max_amount
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(701..801)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("100"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 100 NFTs up");

    // Test that we cannot withdraw again (because of min_delay)
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(601..701)),
            None),
        false);

    // Set a time that is past the min_delay
    set_test_runner_clock(&mut test_runner, 900);

    // Test that we can withdraw again after the min_delay is over
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        TokenQuantity::NonFungible(
            Some(to_nflids(501..601)),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("100"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 100 NFTs up");

    

    // Set up NonFungible allowance spec and use them to withdraw
    // non-fungibles.

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   None,
                                                   0,
                                                   AllowanceLifeCycle::OneOff,
                                                   play_resource,
                                                   Some(TokenQuantity::NonFungible(
                                                       Some([650.into(), 651.into()].into()),
                                                       Some(10))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  None,
                                                  0,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                   Some(TokenQuantity::NonFungible(
                                                       Some([660.into(), 661.into()].into()),
                                                       Some(10))));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  None,
                                                  0,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                   Some(TokenQuantity::NonFungible(
                                                       Some([670.into(), 671.into()].into()),
                                                       Some(10))));

    // Bob gets all these allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(3)));

    // Pull out the named NFTs above so they don't randomly get pulled
    // as the arbitrary part of a withdraw. We will put them back in
    // when they are needed for a test.
    call_withdraw(&mut test_runner,
                  &alice,
                  escrow,
                  &alice_pool_badge,
                  play_resource,
                  TokenQuantity::NonFungible(
                      Some([650.into(), 651.into(),
                            660.into(), 661.into(),
                            670.into(), 671.into()].into()),
                      None));

    call_deposit_funds_with_non_fungibles(&mut test_runner,
                                          &alice,
                                          escrow,
                                          &alice_pool_badge,
                                          None,
                                          play_resource,
                                          [650.into(), 651.into()].into(),
                                          true);

    // Verify that we can't pull 12 randos just because the allowance
    // is for 2 + 10
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            None,
            Some(12)),
        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);

    // Verify that we can't pull 10 randos and a named NFT that isn't
    // named in the allowance
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            Some([652.into()].into()),
            Some(10)),
        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    // Verify that we can pull 12 when two of them are our named ones
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            Some([650.into(), 651.into()].into()),
            Some(10)),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("12"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 12 NFTs up");
    assert!(get_component_nflids(&mut test_runner, bob.account, play_resource)
            .is_superset(&[650.into(), 651.into()].into()),
            "Bob should have the named nflids");
    assert!(!get_component_nflids(&mut test_runner, bob.account, allowance_resaddr)
            .contains(allowance_1off_nfgid.local_id()),
            "This allowance should have been burned");
    drop(receipt);


    
    // Verify accumulating allowance lifecycle
    
    call_deposit_funds_with_non_fungibles(&mut test_runner,
                                          &alice,
                                          escrow,
                                          &alice_pool_badge,
                                          None,
                                          play_resource,
                                          [660.into()].into(),
                                          true);

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some([660.into()].into()),
            Some(10)),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("11"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 11 NFTs up");
    assert!(get_component_nflids(&mut test_runner, bob.account, play_resource)
            .is_superset(&[660.into()].into()),
            "Bob should have the named nflid");
    drop(receipt);

    // Bob gives back #660 to see if he can snag it again (he
    // shouldn't be able to since he spent all his randos and it's no
    // longer in his named allowance)
    call_deposit_funds_with_non_fungibles(&mut test_runner,
                                          &bob,
                                          escrow,
                                          &alice_pool_badge,
                                          None,
                                          play_resource,
                                          [660.into()].into(),
                                          true);
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some([660.into()].into()),
            None),
        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);

    // Bob tries to get a random NFT but he's out of randos
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            None,
            Some(1)),
        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    // Bob tries to get the final NFT #661 but it's not there
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some([661.into()].into()),
            None),
        false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::NonFungibleVaultError(
            NonFungibleVaultError::MissingId(id))) = error {
            *id == 661.into()
        } else {
            false
        }
    });
    drop(receipt);

    // Alice deposits #661 so that Bob can finish up
    call_deposit_funds_with_non_fungibles(&mut test_runner,
                                          &alice,
                                          escrow,
                                          &alice_pool_badge,
                                          None,
                                          play_resource,
                                          [661.into()].into(),
                                          true);

    // And finally Bob gets the last one
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some([661.into()].into()),
            None),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("1"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 1 NFT up");
    assert!(get_component_nflids(&mut test_runner, bob.account, play_resource)
            .is_superset(&[661.into()].into()),
            "Bob should have the named nflid");
    assert!(!get_component_nflids(&mut test_runner, bob.account, allowance_resaddr)
            .contains(allowance_acc_nfgid.local_id()),
            "This allowance should have been burned");
    drop(receipt);


    // Verify repeating allowance lifecycle
    
    call_deposit_funds_with_non_fungibles(&mut test_runner,
                                          &alice,
                                          escrow,
                                          &alice_pool_badge,
                                          None,
                                          play_resource,
                                          [670.into(), 671.into()].into(),
                                          true);

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some([670.into(), 671.into()].into()),
            Some(5)),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("7"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 7 NFTs up");
    assert!(get_component_nflids(&mut test_runner, bob.account, play_resource)
            .is_superset(&[670.into(), 671.into()].into()),
            "Bob should have the named nflids");
    drop(receipt);

    
    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            None,
            Some(10)),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("10"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 10 NFTs up");
    drop(receipt);


    // Verify that Bob can withdraw the named ones again
    call_deposit_funds_with_non_fungibles(&mut test_runner,
                                          &bob,
                                          escrow,
                                          &alice_pool_badge,
                                          None,
                                          play_resource,
                                          [670.into(), 671.into()].into(),
                                          true);

    let receipt = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some([670.into(), 671.into()].into()),
            Some(10)),
        true);
    let result = receipt.expect_commit_ignore_outcome();
    assert_eq!(dec!("12"),
               balance_change_amount(result,
                                     test_runner.get_component_vaults(bob.account, play_resource),
                                     play_resource),
               "Bob should be 12 NFTs up");
    assert!(get_component_nflids(&mut test_runner, bob.account, play_resource)
            .is_superset(&[670.into(), 671.into()].into()),
            "Bob should have the named nflids");
    drop(receipt);
}


#[test]
fn test_withdraw_non_fungibles_with_allowance_fails_outside_validity_period() {
    let (mut test_runner, alice, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &alice, package);

    let badge_res =
        test_runner.create_non_fungible_resource(alice.account);
    let alice_pool_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let play_resource =
        create_nft_resource(&mut test_runner,
                            &alice,
                            1,
                            1000,
                            None);

    // Doing small loops to avoid hitting cost unit limits

    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
    }

    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
    }

    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
    }
    
    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource,
                           dec!("50"),
                           true);
    }


    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   Some(500),
                                                   2,
                                                   AllowanceLifeCycle::OneOff,
                                                   play_resource,
                                                   Some(TokenQuantity::Fungible(dec!("10"))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("10"))));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(TokenQuantity::Fungible(dec!("10"))));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   None,
                                                   2,
                                                   AllowanceLifeCycle::Repeating{
                                                       min_delay: Some(500)},
                                                   play_resource,
                                                   None);

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_acc_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    assert_eq!(allowance_1off_nfgid.resource_address(),
               allowance_rep2_nfgid.resource_address(),
               "Allowances should be of the same NF resource");

    let allowance_resaddr = allowance_1off_nfgid.resource_address();

    let bob = make_user(&mut test_runner, Some(&"Bob".to_owned()));

    // Bob gets all the allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(4)));


    // Test that allowances fail before they are valid

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            Some([1.into()].into()),
            None),
        false);
        
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some([1.into()].into()),
            None),
        false);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some([1.into()].into()),
            None),
        false);


    // Test that allowances fail after they are invalid

    set_test_runner_clock(&mut test_runner, 600);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        TokenQuantity::NonFungible(
            Some([1.into()].into()),
            None),
        false);
        
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        TokenQuantity::NonFungible(
            Some([1.into()].into()),
            None),
        false);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        TokenQuantity::NonFungible(
            Some([1.into()].into()),
            None),
        false);
}

#[test]
fn test_subsidize_with_allowance() {
    let (mut test_runner, alice, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &alice, package);

    let alice_pool_res =
        test_runner.create_non_fungible_resource(alice.account);
    let alice_pool_badge =
        NonFungibleGlobalId::new(alice_pool_res, 1.into());
    let play_resource_alice =
        test_runner.create_fungible_resource(dec!("10000"), 18, alice.account);

    let bob = make_user(&mut test_runner, Some(&"Bob".to_owned()));
    let bob_pool_res =
        test_runner.create_non_fungible_resource(bob.account);
    let bob_pool_badge =
        NonFungibleGlobalId::new(bob_pool_res, 1.into());
    let play_resource_bob =
        test_runner.create_fungible_resource(dec!("10000"), 18, bob.account);


    // Put some XRD in Alice's pool so she can subsidize Bob's
    // playtime. Keep a little for Alice's own tests.
    call_deposit_funds(&mut test_runner,
                       &alice,
                       escrow,
                       &alice_pool_badge,
                       None,
                       XRD,
                       dec!("9000"),
                       true);

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   Some(500),
                                                   2,
                                                   AllowanceLifeCycle::OneOff,
                                                   XRD,
                                                   Some(TokenQuantity::Fungible(dec!("100"))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  XRD,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  XRD,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   None,
                                                   2,
                                                   AllowanceLifeCycle::Repeating{
                                                       min_delay: Some(500)},
                                                   XRD,
                                                   None);

    let allowance_resaddr = allowance_1off_nfgid.resource_address();

    
    // Bob gets all the allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(4)));


    // Verify that allowances fail before they are valid
    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_1off_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2009 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_acc_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2009 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_rep_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2009 ")
        } else {
            false
        }
    });
    drop(receipt);

    // Advance time to be within validity period
    set_test_runner_clock(&mut test_runner, 120);

    // Verify that we cannot overspend the allowances
    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_1off_nfgid.clone(),
                                               dec!("200"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2010 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_acc_nfgid.clone(),
                                               dec!("200"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2010 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_rep_nfgid.clone(),
                                               dec!("200"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2010 ")
        } else {
            false
        }
    });
    drop(receipt);

    
    // Verify that allowances function within their valid period

    // Verify subsidy with one-off allowance
    let result =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_1off_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               true);
    let receipt = result.expect_commit_success();
    assert!(balance_change_amount(&receipt,
                                  test_runner.get_component_vaults(bob.account, XRD),
                                  XRD).is_zero(),
            "Bob should have not paid the XRD fee");
    assert!(balance_change_amount(&receipt,
                                  test_runner.get_component_vaults(escrow, XRD),
                                  XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");
    // (verify that the 1-off allowance is now burned)
    assert!(!get_component_nflids(&mut test_runner, bob.account, allowance_resaddr)
            .contains(allowance_1off_nfgid.local_id()),
            "This allowance should have been burned");
    drop(result);

    // Verify subsidy with accumulating allowance
    for _ in 0..10 { // the 11th would fail
        let result =
            call_subsidize_with_allowance_and_play(&mut test_runner,
                                                   &bob,
                                                   escrow,
                                                   bob_pool_badge.clone(),
                                                   allowance_acc_nfgid.clone(),
                                                   dec!("10"),
                                                   play_resource_bob,
                                                   false,
                                                   true);
        let receipt = result.expect_commit_success();
        assert!(balance_change_amount(&receipt,
                                      test_runner.get_component_vaults(bob.account, XRD),
                                      XRD).is_zero(),
                "Bob should have not paid the XRD fee");
        assert!(balance_change_amount(&receipt,
                                      test_runner.get_component_vaults(escrow, XRD),
                                      XRD)
                != Decimal::ZERO,
                "Escrow should have paid the XRD fee");
        drop(result);
    }
    // (verify that the Allowance is now burned)
    assert!(!get_component_nflids(&mut test_runner, bob.account, allowance_resaddr)
            .contains(allowance_acc_nfgid.local_id()),
            "This allowance should have been burned");

    
    // Verify subsidy with repeating allowance
    for _ in 0..100 {
        let result =
            call_subsidize_with_allowance_and_play(&mut test_runner,
                                                   &bob,
                                                   escrow,
                                                   bob_pool_badge.clone(),
                                                   allowance_rep_nfgid.clone(),
                                                   dec!("10"),
                                                   play_resource_bob,
                                                   false,
                                                   true);
        let receipt = result.expect_commit_success();
        assert!(balance_change_amount(&receipt,
                                      test_runner.get_component_vaults(bob.account, XRD),
                                      XRD).is_zero(),
                "Bob should have not paid the XRD fee");
        assert!(balance_change_amount(&receipt,
                                      test_runner.get_component_vaults(escrow, XRD),
                                      XRD)
                != Decimal::ZERO,
                "Escrow should have paid the XRD fee");
        drop(result);
    }


    // Verify subsidy without max_amount
    let result =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_rep2_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               true);
    let receipt = result.expect_commit_success();
    assert!(balance_change_amount(&receipt,
                                  test_runner.get_component_vaults(bob.account, XRD),
                                  XRD).is_zero(),
            "Bob should have not paid the XRD fee");
    assert!(balance_change_amount(&receipt,
                                  test_runner.get_component_vaults(escrow, XRD),
                                  XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");
    drop(result);

    
    // Verify that we cannot subsidize again (because of min_delay)
    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_rep2_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2009 ")
        } else {
            false
        }
    });
    drop(receipt);


    // Verify that we can subsidize again after the min_delay is over
    set_test_runner_clock(&mut test_runner, 720);

    let result =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_rep2_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               true);
    let receipt = result.expect_commit_success();
    assert!(balance_change_amount(&receipt,
                                  test_runner.get_component_vaults(bob.account, XRD),
                                  XRD).is_zero(),
            "Bob should have not paid the XRD fee");
    assert!(balance_change_amount(&receipt,
                                  test_runner.get_component_vaults(escrow, XRD),
                                  XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");
    drop(result);


    // Recreate those allowances that got burned
    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   &alice_pool_badge,
                                                   Some(500),
                                                   2,
                                                   AllowanceLifeCycle::OneOff,
                                                   XRD,
                                                   Some(TokenQuantity::Fungible(dec!("100"))));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  &alice_pool_badge,
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  XRD,
                                                  Some(TokenQuantity::Fungible(dec!("100"))));

    // And give them to Bob
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                TokenQuantity::Fungible(dec!(2)));

    // Test that allowances fail after they are invalid
    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_1off_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2011 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_acc_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2011 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    let receipt =
        call_subsidize_with_allowance_and_play(&mut test_runner,
                                               &bob,
                                               escrow,
                                               bob_pool_badge.clone(),
                                               allowance_rep_nfgid.clone(),
                                               dec!("10"),
                                               play_resource_bob,
                                               false,
                                               false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2011 ")
        } else {
            false
        }
    });
    drop(receipt);


    
    // Just as a control, verifies that we can do a normal subsidy
    let result =
        call_subsidize_and_play(&mut test_runner,
                                &alice,
                                escrow,
                                &alice_pool_badge,
                                dec!("10"),
                                play_resource_alice);

    assert!(balance_change_amount(&result,
                                  test_runner.get_component_vaults(alice.account, XRD),
                                  XRD).is_zero(),
            "Alice should have not paid the XRD fee");
    assert!(balance_change_amount(&result,
                                  test_runner.get_component_vaults(escrow, XRD),
                                  XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");
}


// Tests the generation of automatic allowance for trusted agents that
// deposit funds into an escrow pool.
#[test]
fn test_automatic_allowance() {
    let (mut test_runner, alice, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &alice, package);

    let alice_pool_res =
        test_runner.create_non_fungible_resource(alice.account);
    let alice_pool_badge =
        NonFungibleGlobalId::new(alice_pool_res, 1.into());

    let play_resource_nf =
        create_nft_resource(&mut test_runner, &alice, 0, 1000, None);
    let play_resource_f =
        test_runner.create_fungible_resource(dec!("10000"), 18, alice.account);

    let bob = make_user(&mut test_runner, Some(&"Bob".to_owned()));
    let bob_id_res_1 =
        test_runner.create_non_fungible_resource(bob.account);
    let bob_id_badge_1_1_trusted =
        NonFungibleGlobalId::new(bob_id_res_1, 1.into());
    let bob_id_badge_1_2_untrusted =
        NonFungibleGlobalId::new(bob_id_res_1, 2.into());
    let bob_id_res_2_trusted =
        test_runner.create_non_fungible_resource(bob.account);
    let bob_id_badge_2_1 =
        NonFungibleGlobalId::new(bob_id_res_2_trusted, 1.into());

    // Put some of Alice's play tokens into her pool
    let receipt =
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           None,
                           play_resource_f,
                           dec!(5000),
                           true);
    // This also creates the pool so let's remember its Allowance
    // resource address.
    let result = receipt.expect_commit_success();
    let alice_pool_allowance_resource = result.new_resource_addresses()[0];
    drop(receipt);
    
    // Pool takes NFTs #0..#499 inclusive
    for n in 0..10 {
        call_deposit_funds_with_non_fungibles(
            &mut test_runner,
            &alice,
            escrow,
            &alice_pool_badge,
            None,
            play_resource_nf,
            (n*50 .. n*50+50).map(|v|v.into()).collect(),
            true);
    }

    // Give the rest to Bob
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &play_resource_f,
                TokenQuantity::Fungible(dec!(5000)));
    // Bob takes NFTs #500..#999 inclusive
    for _ in 0..10 {
        give_tokens(&mut test_runner,
                    &alice.account,
                    &alice.nfgid,
                    &bob.account,
                    &play_resource_nf,
                    TokenQuantity::Fungible(dec!(50)));
    }


    // Verify that without trust yet, Bob can't get allowances

    // (non-trusted nfgid, non-fungible play resource)
    let receipt =
        call_deposit_funds_with_non_fungibles(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_1_1_trusted.clone()),
            play_resource_nf,
            [500.into()].into(),
            false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2013 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    // (non-trusted resource, non-fungible play resource)
    let receipt =
        call_deposit_funds_with_non_fungibles(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_2_1.clone()),
            play_resource_nf,
            [500.into()].into(),
            false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2013 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    // (non-trusted nfgid, fungible play resource)
    let receipt =
        call_deposit_funds(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_1_1_trusted.clone()),
            play_resource_f,
            dec!(10),
            false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2013 ")
        } else {
            false
        }
    });
    drop(receipt);

    // (non-trusted resource, fungible play resource)
    let receipt =
        call_deposit_funds(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_2_1.clone()),
            play_resource_f,
            dec!(10),
            false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2013 ")
        } else {
            false
        }
    });
    drop(receipt);

    
    // Set up Alice's trust in Bob
    call_add_trusted_nfgid(&mut test_runner,
                           &alice,
                           escrow,
                           &alice_pool_badge,
                           bob_id_badge_1_1_trusted.clone(),
                           true);

    call_add_trusted_resource(&mut test_runner,
                              &alice,
                              escrow,
                              &alice_pool_badge,
                              bob_id_res_2_trusted.clone(),
                              true);

    // Verify that untrusted callers still don't get to play
    let receipt =
        call_deposit_funds(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_1_2_untrusted.clone()),
            play_resource_f, //fungible
            dec!(10),
            false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2013 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_deposit_funds(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_1_2_untrusted.clone()),
            play_resource_nf, // non-fungible
            dec!(10),
            false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2013 ")
        } else {
            false
        }
    });
    drop(receipt);

    
    // Verify that trusted callers do get allowances back
    
    // (trusted nfgid, non-fungible play resource)
    let receipt =
        call_deposit_funds_with_non_fungibles(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_1_1_trusted.clone()),
            play_resource_nf,
            [500.into()].into(),
            true);
    let result = receipt.expect_commit_success();
    let allowance = balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.first().unwrap().clone();
    drop(receipt);

    // Try to abuse the allowance
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::Fungible(dec!(1)),
                                     false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::NonFungible(Some([499.into()].into()), None),
                                     false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::NonFungible(Some([499.into()].into()), Some(1)),
                                     false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);

    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::NonFungible(Some([500.into()].into()), Some(1)),
                                     false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2012 ")
        } else {
            false
        }
    });
    drop(receipt);

    // Verify that the allowance is usable
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::NonFungible(Some([500.into()].into()), None),
                                     true);
    let result = receipt.expect_commit_success();
    assert!(balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, play_resource_nf),
        play_resource_nf).0.contains(&500.into()),
            "Bob should now have #500 back");
    assert!(balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.len() == 0,
            "The allowance should now be burned");
    drop(receipt);

    
    // (trusted resource, non-fungible play resource)
    let receipt =
        call_deposit_funds_with_non_fungibles(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_2_1.clone()),
            play_resource_nf,
            [501.into()].into(),
            true);
    let result = receipt.expect_commit_success();
    let allowance = balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.first().unwrap().clone();
    drop(receipt);

    // Verify that the allowance is usable
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::NonFungible(Some([501.into()].into()), None),
                                     true);
    let result = receipt.expect_commit_success();
    assert!(balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, play_resource_nf),
        play_resource_nf).0.contains(&501.into()),
            "Bob should now have #501 back");
    assert!(balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.len() == 0,
            "The allowance should now be burned");
    drop(receipt);
    drop(allowance);

    
    // (trusted nfgid, fungible play resource)
    let receipt =
        call_deposit_funds(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_1_1_trusted.clone()),
            play_resource_f,
            dec!(100),
            true);
    let result = receipt.expect_commit_success();
    let allowance = balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.first().unwrap().clone();
    drop(receipt);

    // Try to abuse the allowance
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::Fungible(dec!(101)),
                                     false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2010 ")
        } else {
            false
        }
    });
    drop(receipt);
    
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::NonFungible(None, Some(101)),
                                     false);
    receipt.expect_specific_failure(|error| {
        if let RuntimeError::ApplicationError(ApplicationError::PanicMessage(msg)) = error {
            msg.starts_with("2010 ")
        } else {
            false
        }
    });
    drop(receipt);

    // Verify that the allowance is usable
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::Fungible(dec!(50)),
                                     true);
    let result = receipt.expect_commit_success();
    assert!(balance_change_amount(
        result,
        test_runner.get_component_vaults(bob.account, play_resource_f),
        play_resource_f) == dec!(50),
            "Bob should now have 50 tokens back");

    // and again, to test burn after multiple uses
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::Fungible(dec!(50)),
                                     true);
    let result = receipt.expect_commit_success();
    assert!(balance_change_amount(
        result,
        test_runner.get_component_vaults(bob.account, play_resource_f),
        play_resource_f) == dec!(50),
            "Bob should now have another 50 tokens back");
    assert!(balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.len() == 0,
            "The allowance should now be burned");
    drop(receipt);

    // (trusted resource, fungible play resource)
    let receipt =
        call_deposit_funds(
            &mut test_runner,
            &bob,
            escrow,
            &alice_pool_badge,
            Some(bob_id_badge_2_1.clone()),
            play_resource_f,
            dec!(100),
            true);
    let result = receipt.expect_commit_success();
    let allowance = balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.first().unwrap().clone();
    drop(receipt);

    // Verify that the allowance is usable
    let receipt =
        call_withdraw_with_allowance(&mut test_runner,
                                     &bob,
                                     escrow,
                                     &NonFungibleGlobalId::new(alice_pool_allowance_resource,
                                                               allowance.clone()),
                                     TokenQuantity::Fungible(dec!(100)),
                                     true);
    let result = receipt.expect_commit_success();
    assert!(balance_change_amount(
        result,
        test_runner.get_component_vaults(bob.account, play_resource_f),
        play_resource_f) == dec!(100),
            "Bob should now have 100 tokens back");
    assert!(balance_change_nflids(
        result,
        test_runner.get_component_vaults(bob.account, alice_pool_allowance_resource),
        alice_pool_allowance_resource).0.len() == 0,
            "The allowance should now be burned");
    drop(receipt);
    drop(allowance);
}
