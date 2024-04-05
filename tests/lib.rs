use std::ops::Neg;

use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use radix_engine::transaction::CommitResult;
use escrow::asking_type::AskingType;

mod common;
use common::{User, setup_for_test,
             balance_change_amount,
             balance_change_nflids,
             make_user,
             give_tokens,
             create_nft_resource,
             to_nflids,
             set_test_runner_clock};
use escrow::{AllowanceLifeCycle, AllowanceNfData};

fn call_instantiate(test_runner: &mut DefaultTestRunner,
                    user: &User,
                    package: PackageAddress)
                    -> ComponentAddress
{
    let manifest = ManifestBuilder::new()
        .call_function(package,
                       "Escrow",
                       "instantiate_escrow",
                       manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().new_component_addresses()[0]
}

fn call_deposit_funds(test_runner: &mut DefaultTestRunner,
                      user: &User,
                      escrow: ComponentAddress,
                      owner: NonFungibleGlobalId,
                      resource: ResourceAddress,
                      amount: Decimal) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(user.account,
                               resource,
                               amount)
        .take_from_worktop(
            resource,
            amount,
            "funds_bucket")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(owner,
                                    lookup.bucket("funds_bucket"),
                                    None::<ManifestProof>))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().clone()
}

fn call_read_funds(test_runner: &mut DefaultTestRunner,
                   user: &User,
                   escrow: ComponentAddress,
                   owner: NonFungibleGlobalId,
                   resource: ResourceAddress)
                   -> Decimal
{
    let manifest = ManifestBuilder::new()
        .call_method(escrow,
                     "read_funds",
                     manifest_args!(owner, resource))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().output(1)
}

fn call_withdraw(test_runner: &mut DefaultTestRunner,
                 user: &User,
                 escrow: ComponentAddress,
                 caller: NonFungibleGlobalId,
                 resource: ResourceAddress,
                 amount: Decimal) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "withdraw",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    resource,
                                    amount))
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().clone()
}

fn call_withdraw_all_of(test_runner: &mut DefaultTestRunner,
                        user: &User,
                        escrow: ComponentAddress,
                        caller: NonFungibleGlobalId,
                        resource: ResourceAddress) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "withdraw_all_of",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    resource))
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().clone()
}

fn call_withdraw_non_fungibles(test_runner: &mut DefaultTestRunner,
                               user: &User,
                               escrow: ComponentAddress,
                               caller: NonFungibleGlobalId,
                               resource: ResourceAddress,
                               nflids: BTreeSet<NonFungibleLocalId>) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "withdraw_non_fungibles",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    resource,
                                    nflids))
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    receipt.expect_commit_success().clone()
}

fn call_subsidize_and_play(test_runner: &mut DefaultTestRunner,
                           user: &User,
                           escrow: ComponentAddress,
                           caller: NonFungibleGlobalId,
                           amount: Decimal,
                           play_resource: ResourceAddress) -> CommitResult
{
    call_subsidize_and_play_impl(test_runner,
                                 user,
                                 escrow,
                                 caller,
                                 amount,
                                 play_resource,
                                 "subsidize",
                                 false)
}

fn call_subsidize_contingent_and_play(test_runner: &mut DefaultTestRunner,
                                      user: &User,
                                      escrow: ComponentAddress,
                                      caller: NonFungibleGlobalId,
                                      amount: Decimal,
                                      play_resource: ResourceAddress) -> CommitResult
{
    call_subsidize_and_play_impl(test_runner,
                                 user,
                                 escrow,
                                 caller,
                                 amount,
                                 play_resource,
                                 "subsidize_contingent",
                                 false)
}

fn call_subsidize_contingent_and_fail(test_runner: &mut DefaultTestRunner,
                                      user: &User,
                                      escrow: ComponentAddress,
                                      caller: NonFungibleGlobalId,
                                      amount: Decimal,
                                      play_resource: ResourceAddress) -> CommitResult
{
    call_subsidize_and_play_impl(test_runner,
                                 user,
                                 escrow,
                                 caller,
                                 amount,
                                 play_resource,
                                 "subsidize_contingent",
                                 true)
}

fn call_subsidize_and_play_impl(test_runner: &mut DefaultTestRunner,
                                user: &User,
                                escrow: ComponentAddress,
                                caller: NonFungibleGlobalId,
                                amount: Decimal,
                                play_resource: ResourceAddress,
                                method_name: &str,
                                fail: bool) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .lock_fee(user.account, dec!("10"))
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            method_name,
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    amount))

        // Now some busywork to build up a bit of fees
        .withdraw_from_account(user.account,
                               play_resource,
                               dec!("10"))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds1")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds1"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds2")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds2"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds3")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds3"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds4")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds4"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds5")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds5"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds6")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds6"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds7")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds7"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds8")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds8"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            play_resource,
            dec!("1"),
            "play_funds9")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds9"),
                                    None::<ManifestProof>))
        .take_from_worktop(
            // Using a ridiculously large amount to force a fail
            play_resource,
            if fail { dec!("10000") } else { dec!("1") },
            "play_funds10")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds10"),
                                    None::<ManifestProof>))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() == fail {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    if fail {
        receipt.expect_commit_failure().clone()
    } else {
        receipt.expect_commit_success().clone()
    }
}

fn call_mint_allowance(test_runner: &mut DefaultTestRunner,
                       user: &User,
                       escrow: ComponentAddress,
                       caller: NonFungibleGlobalId,
                       valid_until: Option<i64>,
                       valid_after: i64,
                       life_cycle: AllowanceLifeCycle,
                       for_resource: ResourceAddress,
                       max_amount: Option<Decimal>) -> NonFungibleGlobalId
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "mint_allowance",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    valid_until,
                                    valid_after,
                                    life_cycle,
                                    for_resource,
                                    max_amount))
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }

    let result = receipt.expect_commit_success().clone();

    // This only works because we know there is exactly one non-XRD
    // balance change and it's the one we're interested in.
    let allowance_resaddr = result.vault_balance_changes()
        .iter()
        .filter(|(_, (res, _))| *res != XRD)
        .collect::<Vec<_>>()[0].1.0;
    
    let (added, _) = balance_change_nflids(
        &result,
        test_runner.get_component_vaults(user.account, allowance_resaddr),
        allowance_resaddr);

    NonFungibleGlobalId::new(allowance_resaddr, added.first().unwrap().clone())
}

fn call_reduce_allowance(test_runner: &mut DefaultTestRunner,
                         user: &User,
                         escrow: ComponentAddress,
                         allowance: NonFungibleGlobalId,
                         new_max: Decimal)
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "reduce_allowance",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    new_max))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if !receipt.is_commit_success() {
        println!("{:?}", receipt);
        panic!("TRANSACTION FAIL");
    }
}

fn call_withdraw_with_allowance(test_runner: &mut DefaultTestRunner,
                                user: &User,
                                escrow: ComponentAddress,
                                allowance: &NonFungibleGlobalId,
                                amount: Decimal,
                                succeed: bool) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .withdraw_non_fungibles_from_account(
            user.account,
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]))
        .take_non_fungibles_from_worktop(
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]),
            "allowance_bucket")
        .call_method_with_name_lookup(
            escrow,
            "withdraw_with_allowance",
            |lookup| manifest_args!(lookup.bucket("allowance_bucket"),
                                    amount))
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != succeed {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    if succeed {
        receipt.expect_commit_success().clone()
    } else {
        receipt.expect_commit_failure().clone()
    }
}

fn call_withdraw_non_fungibles_with_allowance(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    escrow: ComponentAddress,
    allowance: &NonFungibleGlobalId,
    nflids: BTreeSet<NonFungibleLocalId>,
    succeed: bool) -> CommitResult
{
    let manifest = ManifestBuilder::new()
        .withdraw_non_fungibles_from_account(
            user.account,
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]))
        .take_non_fungibles_from_worktop(
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]),
            "allowance_bucket")
        .call_method_with_name_lookup(
            escrow,
            "withdraw_non_fungibles_with_allowance",
            |lookup| manifest_args!(lookup.bucket("allowance_bucket"),
                                    nflids))
        .deposit_batch(user.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != succeed {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    if succeed {
        receipt.expect_commit_success().clone()
    } else {
        receipt.expect_commit_failure().clone()
    }
}

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
    let result =
        call_deposit_funds(&mut test_runner,
                           &owner,
                           escrow,
                           owner_badge.clone(),
                           XRD,
                           dec!("100"));

    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD),
               "Component should be up 100 XRD");


    // The second time, our existing pool is reused
    let result =
        call_deposit_funds(&mut test_runner,
                           &owner,
                           escrow,
                           owner_badge.clone(),
                           XRD,
                           dec!("50"));

    assert_eq!(dec!("50"),
               balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD),
               "Component should be up 50 XRD");

    assert_eq!(dec!("150"),
               call_read_funds(&mut test_runner,
                               &owner,
                               escrow,
                               owner_badge.clone(),
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
                       owner_badge.clone(),
                       XRD,
                       dec!("100"));

    let result =
        call_withdraw(&mut test_runner,
                      &owner,
                      escrow,
                      owner_badge.clone(),
                      XRD,
                      dec!("10"));

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
        test_runner.create_non_fungible_resource(owner.account);

    call_deposit_funds(&mut test_runner,
                       &owner,
                       escrow,
                       owner_badge.clone(),
                       nfts_res,
                       dec!("3"));

    let result =
        call_withdraw_non_fungibles(&mut test_runner,
                                    &owner,
                                    escrow,
                                    owner_badge.clone(),
                                    nfts_res,
                                    [1.into(), 3.into()].into());

    assert_eq!(dec!("-2"),
               balance_change_amount(&result, test_runner.get_component_vaults(escrow, nfts_res), nfts_res),
               "Escrow should be down 2 NFTs");
    assert_eq!(dec!("2"),
               balance_change_amount(&result, test_runner.get_component_vaults(owner.account, nfts_res), nfts_res),
               "User should be up 2 NFTs");
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
                       owner_badge.clone(),
                       XRD,
                       dec!("100"));

    let result =
        call_withdraw_all_of(&mut test_runner,
                             &owner,
                             escrow,
                             owner_badge.clone(),
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
                       owner_badge.clone(),
                       nfts_res,
                       dec!("3"));

    let result =
        call_withdraw_all_of(&mut test_runner,
                             &owner,
                             escrow,
                             owner_badge.clone(),
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
                       owner_badge.clone(),
                       XRD,
                       dec!("100"));
    
    let result =
        call_subsidize_and_play(&mut test_runner,
                                &owner,
                                escrow,
                                owner_badge.clone(),
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
                       owner_badge.clone(),
                       XRD,
                       dec!("100"));


    // Test contingent fee on a successful tx manifest
    
    let result =
        call_subsidize_contingent_and_play(&mut test_runner,
                                           &owner,
                                           escrow,
                                           owner_badge.clone(),
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
                                           owner_badge.clone(),
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
                                              owner_badge.clone(),
                                              Some(50),
                                              2,
                                              AllowanceLifeCycle::Accumulating,
                                              XRD,
                                              Some(dec!("100")));

    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_nfgid.resource_address(),
        allowance_nfgid.local_id().clone());
//    let nfdata = get_non_fungible_data::<AllowanceNfData>(
//        test_runner.substate_db(),
//        allowance_nfgid.resource_address(),
//        allowance_nfgid.local_id().clone());

    assert_eq!(escrow, nfdata.escrow_pool.0,
               "allowance should reference correct escrow instance");
    assert_eq!(owner_badge, nfdata.escrow_pool.1,
               "owner should own the allowance's pool");
    assert_eq!(Some(50), nfdata.valid_until,
               "valid_until should be as we set it");
    assert_eq!(2, nfdata.valid_after,
               "valid_after should be as we set it");
    assert!(matches!(nfdata.life_cycle, AllowanceLifeCycle::Accumulating),
            "life_cycle should be as we set it");
    assert_eq!(XRD, nfdata.for_resource,
               "for_resource should be as we set it");
    assert_eq!(Some(dec!("100")), nfdata.max_amount,
               "max_amount should be as we set it");
}


#[test]
fn test_reduce_allowance() {
    let (mut test_runner, owner, package) = setup_for_test();

    let escrow = call_instantiate(&mut test_runner, &owner, package);

    let badge_res =
        test_runner.create_non_fungible_resource(owner.account);
    let owner_badge =
        NonFungibleGlobalId::new(badge_res, 1.into());

    let allowance_nfgid = call_mint_allowance(&mut test_runner,
                                              &owner,
                                              escrow,
                                              owner_badge.clone(),
                                              Some(50),
                                              2,
                                              AllowanceLifeCycle::Accumulating,
                                              XRD,
                                              Some(dec!("100")));

    call_reduce_allowance(&mut test_runner,
                          &owner,
                          escrow,
                          allowance_nfgid.clone(),
                          dec!("25"));

    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_nfgid.resource_address(),
        allowance_nfgid.local_id().clone());

//    let nfdata = get_non_fungible_data::<AllowanceNfData>(
//        test_runner.substate_db(),
//        allowance_nfgid.resource_address(),
//        allowance_nfgid.local_id().clone());

    assert_eq!(Some(dec!("25")), nfdata.max_amount,
               "max_amount should have been reduced");
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
                       alice_pool_badge.clone(),
                       play_resource,
                       dec!("10000"));

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::OneOff,
                                                  play_resource,
                                                  Some(dec!("100")));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(dec!("100")));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(dec!("100")));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
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
                AskingType::Fungible(dec!(4)));


    // Test allowances within their valid period

    set_test_runner_clock(&mut test_runner, 400);

    // Try to withdraw too much
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        dec!("10000"),
        false);
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        dec!("10000"),
        false);
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        dec!("10000"),
        false);


    // Test withdrawal with one-off allowance
    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        dec!("55"),
        true);

    assert_eq!(dec!("55"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 55 funds up");
    assert_eq!(dec!("-1"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, allowance_resaddr), allowance_resaddr),
               "Bob's one-off allowance should be burnt");


    // Test withdrawal with accumulating allowance
    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        dec!("40"),
        true);

    assert_eq!(dec!("40"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 40 funds up");
    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_resaddr,
        allowance_acc_nfgid.local_id().clone());
//    let nfdata =
//        get_non_fungible_data::<AllowanceNfData>(
//            test_runner.substate_db(),
//            allowance_resaddr,
//            allowance_acc_nfgid.local_id().clone());
    assert_eq!(dec!("60"),
               nfdata.max_amount.unwrap(),
               "Accumulating allowance should be down 40 tokens");

    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        dec!("60"),
        true);
    assert_eq!(dec!("60"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 60 funds up");
    assert_eq!(dec!("-1"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, allowance_resaddr), allowance_resaddr),
               "Bob's accumulating allowance should be burnt");


    // Test withdrawal with repeating allowance
    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        dec!("10"),
        true);
    assert_eq!(dec!("10"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 10 funds up");

    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        dec!("100"),
        true);
    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 100 funds up");

    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        dec!("100"),
        true);
    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 100 funds up");


    // Test withdrawal without max_amount
    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        dec!("1000"),
        true);
    assert_eq!(dec!("1000"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 1000 funds up");

    // Test that we cannot withdraw again (because of min_delay)
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        dec!("100"),
        false);

    // Set a time that is past the min_delay
    set_test_runner_clock(&mut test_runner, 900);

    // Test that we can withdraw again after the min_delay is over
    let result = call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        dec!("100"),
        true);
    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
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
                       alice_pool_badge.clone(),
                       play_resource,
                       dec!("10000"));

    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::OneOff,
                                                  play_resource,
                                                  Some(dec!("100")));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(dec!("100")));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(dec!("100")));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
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
                AskingType::Fungible(dec!(4)));


    // Test that allowances fail before they are valid

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        dec!("1"),
        false);
        
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        dec!("1"),
        false);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        dec!("1"),
        false);


    // Test that allowances fail after they are invalid

    set_test_runner_clock(&mut test_runner, 600);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        dec!("1"),
        false);
        
    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        dec!("1"),
        false);

    call_withdraw_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        dec!("1"),
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
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
    }


    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
                                                   Some(500),
                                                   2,
                                                   AllowanceLifeCycle::OneOff,
                                                   play_resource,
                                                   Some(dec!("10")));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(dec!("10")));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(dec!("10")));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
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
                AskingType::Fungible(dec!(4)));

    // Test allowances within their valid period

    set_test_runner_clock(&mut test_runner, 400);

    // Try to withdraw too much
    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        to_nflids(1..20).into(),
        false);
    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        to_nflids(1..20).into(),
        false);
    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        to_nflids(1..20).into(),
        false);


    // Test withdrawal with one-off allowance
    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        to_nflids(1..6).into(),
        true);

    assert_eq!(dec!("5"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 5 NFTs up");
    assert_eq!(dec!("-1"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, allowance_resaddr), allowance_resaddr),
               "Bob's one-off allowance should be burnt");


    // Test withdrawal with accumulating allowance
    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        to_nflids(11..15).into(),
        true);

    assert_eq!(dec!("4"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 4 NFTs up");
    let nfdata = test_runner.get_non_fungible_data::<AllowanceNfData>(
        allowance_resaddr,
        allowance_acc_nfgid.local_id().clone());
//    let nfdata =
//        get_non_fungible_data::<AllowanceNfData>(
//            test_runner.substate_db(),
//            allowance_resaddr,
//            allowance_acc_nfgid.local_id().clone());
    assert_eq!(dec!("6"),
               nfdata.max_amount.unwrap(),
               "Accumulating allowance should be down 4 NFTs");

    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        to_nflids(15..21).into(),
        true);
    assert_eq!(dec!("6"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 6 NFTs up");
    assert_eq!(dec!("-1"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, allowance_resaddr), allowance_resaddr),
               "Bob's accumulating allowance should be burnt");


    // Test withdrawal with repeating allowance
    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        to_nflids(801..805).into(),
        true);
    assert_eq!(dec!("4"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 4 NFTs up");

    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        to_nflids(811..821).into(),
        true);
    assert_eq!(dec!("10"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 10 NFTs up");

    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        to_nflids(821..831).into(),
        true);
    assert_eq!(dec!("10"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 10 NFTs up");


    // Test withdrawal without max_amount
    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        to_nflids(701..801).into(),
        true);
    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 100 NFTs up");

    // Test that we cannot withdraw again (because of min_delay)
    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        to_nflids(601..701).into(),
        false);

    // Set a time that is past the min_delay
    set_test_runner_clock(&mut test_runner, 900);

    // Test that we can withdraw again after the min_delay is over
    let result = call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep2_nfgid,
        to_nflids(501..601).into(),
        true);
    assert_eq!(dec!("100"),
               balance_change_amount(&result, test_runner.get_component_vaults(bob.account, play_resource), play_resource),
               "Bob should be 100 NFTs up");
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
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
    }

    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
    }

    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
    }
    
    for _ in 0..5 {
        call_deposit_funds(&mut test_runner,
                           &alice,
                           escrow,
                           alice_pool_badge.clone(),
                           play_resource,
                           dec!("50"));
    }


    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
                                                   Some(500),
                                                   2,
                                                   AllowanceLifeCycle::OneOff,
                                                   play_resource,
                                                   Some(dec!("10")));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(dec!("10")));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(dec!("10")));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
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
                AskingType::Fungible(dec!(4)));


    // Test that allowances fail before they are valid

    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        [1.into()].into(),
        false);
        
    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        [1.into()].into(),
        false);

    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        [1.into()].into(),
        false);


    // Test that allowances fail after they are invalid

    set_test_runner_clock(&mut test_runner, 600);

    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_1off_nfgid,
        [1.into()].into(),
        false);
        
    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_acc_nfgid,
        [1.into()].into(),
        false);

    call_withdraw_non_fungibles_with_allowance(
        &mut test_runner,
        &bob,
        escrow,
        &allowance_rep_nfgid,
        [1.into()].into(),
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

    let play_resource =
        test_runner.create_fungible_resource(dec!("1000"), 18, alice.account);
    
    call_deposit_funds(&mut test_runner,
                       &alice,
                       escrow,
                       alice_pool_badge.clone(),
                       XRD,
                       dec!("1000"));
    
    let allowance_1off_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::OneOff,
                                                  play_resource,
                                                  Some(dec!("100")));

    let allowance_acc_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Accumulating,
                                                  play_resource,
                                                  Some(dec!("100")));
    
    let allowance_rep_nfgid = call_mint_allowance(&mut test_runner,
                                                  &alice,
                                                  escrow,
                                                  alice_pool_badge.clone(),
                                                  Some(500),
                                                  2,
                                                  AllowanceLifeCycle::Repeating{
                                                      min_delay: None},
                                                  play_resource,
                                                  Some(dec!("100")));

    // This one has a min_delay, no max_amount and no valid_until to
    // test those things
    let allowance_rep2_nfgid = call_mint_allowance(&mut test_runner,
                                                   &alice,
                                                   escrow,
                                                   alice_pool_badge.clone(),
                                                   None,
                                                   2,
                                                   AllowanceLifeCycle::Repeating{
                                                       min_delay: Some(500)},
                                                   play_resource,
                                                   None);

    let allowance_resaddr = allowance_1off_nfgid.resource_address();

    let bob = make_user(&mut test_runner, Some(&"Bob".to_owned()));

    // Bob gets all the allowances from Alice
    give_tokens(&mut test_runner,
                &alice.account,
                &alice.nfgid,
                &bob.account,
                &allowance_resaddr,
                AskingType::Fungible(dec!(4)));

    // TODO the rest below

    // Test that allowances fail before they are valid
    

    // Test that allowances fail after they are invalid


    // Test allowances within their valid period

    // Try to spend too much
    
    // Test subsidy with one-off allowance

    // Test subsidy with accumulating allowance
    
    // Test subsidy with repeating allowance

    // Test subdisy without max_amount

    // Test that we cannot subsidize again (because of min_delay)
    // Test that we can subsidize again after the min_delay is over

    let result =
        call_subsidize_and_play(&mut test_runner,
                                &alice,
                                escrow,
                                alice_pool_badge.clone(),
                                dec!("10"),
                                play_resource);

//    println!("FEE SUMMARY: {:?}", result.fee_summary);
    assert!(balance_change_amount(&result, test_runner.get_component_vaults(alice.account, XRD), XRD).is_zero(),
            "Alice should have not paid the XRD fee");
    assert!(balance_change_amount(&result, test_runner.get_component_vaults(escrow, XRD), XRD)
            != Decimal::ZERO,
            "Escrow should have paid the XRD fee");
}
