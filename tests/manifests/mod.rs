use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use radix_engine::transaction::{CommitResult, TransactionReceipt};
use escrow::token_quantity::TokenQuantity;
use escrow::AllowanceLifeCycle;

use crate::common::*;

pub fn call_instantiate(test_runner: &mut DefaultTestRunner,
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

pub fn call_deposit_funds(test_runner: &mut DefaultTestRunner,
                      user: &User,
                      escrow: ComponentAddress,
                      owner: &NonFungibleGlobalId,
                      trusted_badge: Option<NonFungibleGlobalId>,
                      resource: ResourceAddress,
                      amount: Decimal,
                      expect_success: bool) -> TransactionReceipt
{
    let mut builder = ManifestBuilder::new();
    if let Some(trusted_badge) = &trusted_badge {
        builder = builder
            .create_proof_from_account_of_non_fungibles(
                user.account,
                trusted_badge.resource_address(),
                BTreeSet::from([trusted_badge.local_id().clone()]))
            .pop_from_auth_zone("trusted_badge_proof")
    }
    let manifest = builder
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
                                    if trusted_badge.is_some() {
                                        Some(lookup.proof("trusted_badge_proof"))
                                    } else { None }))
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

pub fn call_deposit_funds_with_non_fungibles(test_runner: &mut DefaultTestRunner,
                                         user: &User,
                                         escrow: ComponentAddress,
                                         owner: &NonFungibleGlobalId,
                                         trusted_badge: Option<NonFungibleGlobalId>,
                                         resource: ResourceAddress,
                                         nflids: BTreeSet<NonFungibleLocalId>,
                                         expect_success: bool) -> TransactionReceipt
{
    let mut manifest = ManifestBuilder::new();
    if let Some(trusted_badge) = &trusted_badge {
        manifest = manifest
            .create_proof_from_account_of_non_fungibles(
                user.account,
                trusted_badge.resource_address(),
                BTreeSet::from([trusted_badge.local_id().clone()]))
            .pop_from_auth_zone("trusted_badge_proof")
    }
    let manifest = manifest
        .withdraw_non_fungibles_from_account(user.account,
                                             resource,
                                             nflids)
        .take_all_from_worktop(
            resource,
            "funds_bucket")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(owner,
                                    lookup.bucket("funds_bucket"),
                                    if trusted_badge.is_some() {
                                        Some(lookup.proof("trusted_badge_proof"))
                                    } else { None }))
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

pub fn call_read_funds(
    test_runner: &mut DefaultTestRunner,
    user: &User,
    escrow: ComponentAddress,
    owner: &NonFungibleGlobalId,
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

pub fn call_withdraw(test_runner: &mut DefaultTestRunner,
                 user: &User,
                 escrow: ComponentAddress,
                 caller: &NonFungibleGlobalId,
                 resource: ResourceAddress,
                 quantity: TokenQuantity) -> CommitResult
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
                                    quantity))
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

pub fn call_withdraw_all_of(test_runner: &mut DefaultTestRunner,
                        user: &User,
                        escrow: ComponentAddress,
                        caller: &NonFungibleGlobalId,
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

pub fn call_subsidize_and_play(test_runner: &mut DefaultTestRunner,
                           user: &User,
                           escrow: ComponentAddress,
                           caller: &NonFungibleGlobalId,
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

pub fn call_subsidize_contingent_and_play(test_runner: &mut DefaultTestRunner,
                                      user: &User,
                                      escrow: ComponentAddress,
                                      caller: &NonFungibleGlobalId,
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

pub fn call_subsidize_contingent_and_fail(test_runner: &mut DefaultTestRunner,
                                      user: &User,
                                      escrow: ComponentAddress,
                                      caller: &NonFungibleGlobalId,
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

pub fn call_subsidize_and_play_impl(test_runner: &mut DefaultTestRunner,
                                user: &User,
                                escrow: ComponentAddress,
                                caller: &NonFungibleGlobalId,
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

pub fn call_subsidize_with_allowance_and_play(test_runner: &mut DefaultTestRunner,
                                          user: &User,
                                          escrow: ComponentAddress,
                                          caller: NonFungibleGlobalId,
                                          allowance: NonFungibleGlobalId,
                                          amount: Decimal,
                                          play_resource: ResourceAddress,
                                          force_fail: bool,
                                          expect_success: bool) -> TransactionReceipt
{
    assert!(!(force_fail && expect_success),
            "don't expect a forced fail to be a successful tx result");
    
    let manifest = ManifestBuilder::new()
        .lock_fee(user.account, dec!("10"))
        .withdraw_non_fungibles_from_account(
            user.account,
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]))
        .take_all_from_worktop(
            allowance.resource_address(),
            "allowance_bucket")
        .call_method_with_name_lookup(
            escrow,
            "subsidize_with_allowance",
            |lookup| manifest_args!(lookup.bucket("allowance_bucket"),
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
            if force_fail { dec!("1000000") } else { dec!("1") },
            "play_funds10")
        .call_method_with_name_lookup(
            escrow,
            "deposit_funds",
            |lookup| manifest_args!(caller.clone(),
                                    lookup.bucket("play_funds10"),
                                    None::<ManifestProof>))

        .deposit_batch(user.account) // returns the Allowance
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

pub fn call_mint_allowance(test_runner: &mut DefaultTestRunner,
                       user: &User,
                       escrow: ComponentAddress,
                       caller: &NonFungibleGlobalId,
                       valid_until: Option<i64>,
                       valid_from: i64,
                       life_cycle: AllowanceLifeCycle,
                       for_resource: ResourceAddress,
                       max_amount: Option<TokenQuantity>) -> NonFungibleGlobalId
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
                                    valid_from,
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

pub fn call_reduce_allowance_to_amount(test_runner: &mut DefaultTestRunner,
                                   user: &User,
                                   escrow: ComponentAddress,
                                   allowance: NonFungibleGlobalId,
                                   new_max: Decimal,
                                   expect_success: bool) -> TransactionReceipt
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "reduce_allowance_to_amount",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    new_max))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
        );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

pub fn call_reduce_allowance_by_nflids(test_runner: &mut DefaultTestRunner,
                                   user: &User,
                                   escrow: ComponentAddress,
                                   allowance: NonFungibleGlobalId,
                                   to_remove: IndexSet<NonFungibleLocalId>,
                                   expect_success: bool) -> TransactionReceipt
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            allowance.resource_address(),
            BTreeSet::from([allowance.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "reduce_allowance_by_nflids",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    to_remove))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
        );

    if receipt.is_commit_success() != expect_success {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    receipt
}

pub fn call_withdraw_with_allowance(test_runner: &mut DefaultTestRunner,
                                user: &User,
                                escrow: ComponentAddress,
                                allowance: &NonFungibleGlobalId,
                                quantity: TokenQuantity,
                                succeed: bool) -> TransactionReceipt
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
                                    quantity))
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
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
    receipt.clone()
}

pub fn call_add_trusted_resource(test_runner: &mut DefaultTestRunner,
                             user: &User,
                             escrow: ComponentAddress,
                             caller: &NonFungibleGlobalId,
                             resource: ResourceAddress,
                             succeed: bool) -> TransactionReceipt
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "add_trusted_resource",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    resource))
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
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
    receipt.clone()
}

pub fn call_remove_trusted_resource(test_runner: &mut DefaultTestRunner,
                                user: &User,
                                escrow: ComponentAddress,
                                caller: &NonFungibleGlobalId,
                                resource: ResourceAddress,
                                succeed: bool) -> TransactionReceipt
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "remove_trusted_resource",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    resource))
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
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
    receipt.clone()
}

pub fn call_is_resource_trusted(test_runner: &mut DefaultTestRunner,
                            user: &User,
                            escrow: ComponentAddress,
                            resource: ResourceAddress,
                            succeed: bool) -> (TransactionReceipt, Option<bool>)
{
    let manifest = ManifestBuilder::new()
        .call_method(
            escrow,
            "is_resource_trusted",
            manifest_args!(resource))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != succeed {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    let answer;
    if succeed {
        let result = receipt.expect_commit_success();
        answer = Some(result.output(1));
    } else {
        receipt.expect_commit_failure();
        answer = None;
    }
    (receipt.clone(), answer)
}

pub fn call_add_trusted_nfgid(test_runner: &mut DefaultTestRunner,
                          user: &User,
                          escrow: ComponentAddress,
                          caller: &NonFungibleGlobalId,
                          nfgid: NonFungibleGlobalId,
                          succeed: bool) -> TransactionReceipt
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "add_trusted_nfgid",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    nfgid))
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
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
    receipt.clone()
}

pub fn call_remove_trusted_nfgid(test_runner: &mut DefaultTestRunner,
                             user: &User,
                             escrow: ComponentAddress,
                             caller: &NonFungibleGlobalId,
                             nfgid: NonFungibleGlobalId,
                             succeed: bool) -> TransactionReceipt
{
    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            user.account,
            caller.resource_address(),
            BTreeSet::from([caller.local_id().clone()]))
        .pop_from_auth_zone("caller_proof")
        .call_method_with_name_lookup(
            escrow,
            "remove_trusted_nfgid",
            |lookup| manifest_args!(lookup.proof("caller_proof"),
                                    nfgid))
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
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
    receipt.clone()
}

pub fn call_is_nfgid_trusted(test_runner: &mut DefaultTestRunner,
                         user: &User,
                         escrow: ComponentAddress,
                         nfgid: NonFungibleGlobalId,
                         succeed: bool) -> (TransactionReceipt, Option<bool>)
{
    let manifest = ManifestBuilder::new()
        .call_method(
            escrow,
            "is_nfgid_trusted",
            manifest_args!(nfgid))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user.pubkey)],
    );

    if receipt.is_commit_success() != succeed {
        println!("{:?}", receipt);
        panic!("TRANSACTION BAD");
    }

    let answer;
    if succeed {
        let result = receipt.expect_commit_success();
        answer = Some(result.output(1));
    } else {
        receipt.expect_commit_failure();
        answer = None;
    }
    (receipt.clone(), answer)
}
