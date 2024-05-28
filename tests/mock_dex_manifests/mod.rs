use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use radix_engine::transaction::{TransactionReceipt};
use scrypto::prelude::*;
use crate::common::*;


pub fn instantiate_mock_dex(test_runner: &mut DefaultTestRunner,
                            owner: &User,
                            package: PackageAddress,
                            meme_resource: ResourceAddress)
    -> ComponentAddress
{
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

    receipt.expect_commit_success().new_component_addresses()[0]
}

pub fn call_limit_sell_direct(
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

pub fn call_limit_buy_direct(
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

pub fn call_market_buy_direct(
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

pub fn call_market_sell_direct(
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


pub fn call_limit_buy_with_escrow(
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

pub fn call_limit_sell_with_escrow(
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
