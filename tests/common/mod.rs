//! Contains common library functions for the test code.

use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::prelude::*;
use radix_engine::transaction::{BalanceChange, CommitResult};
use radix_engine_common::ManifestSbor;
use escrow::asking_type::AskingType;

#[derive(ScryptoSbor, ManifestSbor, NonFungibleData)]
pub struct NfData {
}

#[derive(Clone)]
pub struct User {
    pub pubkey: Secp256k1PublicKey,
    pub account: ComponentAddress,
    pub nfgid: NonFungibleGlobalId,
    pub display_name: Option<String>,
    pub trading_nft: NonFungibleGlobalId,
    pub fake_trading_nft: NonFungibleGlobalId,
}

impl User {
    const EMPTY: &'static str = "";
    pub fn display_name(&self) -> &str {
        if let Some(str) = &self.display_name {
            return &str
        }
        Self::EMPTY
    }
}

pub fn make_user(test_runner: &mut DefaultTestRunner,
                 display_name: Option<&str>) -> User {
    let (user_pubk, _, user_account) =
        test_runner.new_allocated_account();

    let nft_res = test_runner.create_non_fungible_resource(user_account);
    let trading_nft = NonFungibleGlobalId::new(nft_res, 1.into());
    let fake_trading_nft = NonFungibleGlobalId::new(nft_res, 2.into());

    User {
        nfgid: NonFungibleGlobalId::from_public_key(&user_pubk),
        pubkey: user_pubk,
        account: user_account,
        display_name: display_name.map(|v| v.to_string()),
        trading_nft,
        fake_trading_nft,
    }
}

static mut ROUND_COUNTER: u64 = 2;

/// Changes the test runner clock.
pub fn set_test_runner_clock(
    test_runner: &mut DefaultTestRunner,
    time_secs: i64) {
    unsafe {
        test_runner.advance_to_round_at_timestamp(
            Round::of(ROUND_COUNTER), // must be incremented each time
            time_secs * 1000,
        );
        ROUND_COUNTER += 1;
    }
}

pub fn balance_change_amount(commit_result: &CommitResult,
                             vaults: Vec<NodeId>,
                             resource: ResourceAddress)
                      -> Decimal {
    for (_, (vault_id, (vault_resource, delta))) in commit_result.vault_balance_changes().iter().enumerate() {
        if resource == *vault_resource && vaults.contains(vault_id) {
            match delta {
                BalanceChange::Fungible(d) => return *d,
                BalanceChange::NonFungible { added, removed } => {
                    return Decimal::from(added.len() as i64 - removed.len() as i64)
                }
            }
        }
    }
    return Decimal::ZERO;
}

pub fn balance_change_nflids(commit_result: &CommitResult,
                             vaults: Vec<NodeId>,
                             resource: ResourceAddress)
                             -> (BTreeSet<NonFungibleLocalId>, BTreeSet<NonFungibleLocalId>) {
    for (_, (vault_id, (vault_resource, delta))) in commit_result.vault_balance_changes()
        .iter().enumerate()
    {
        if resource == *vault_resource && vaults.contains(vault_id) {
            match delta {
                BalanceChange::NonFungible { added, removed } => {
                    return (added.clone(), removed.clone())
                }
                BalanceChange::Fungible(_) => {},
            }
        }
    }
    return (BTreeSet::new(), BTreeSet::new())
}

//pub fn get_component_balance_change(result: &CommitResult,
//                                    component: &ComponentAddress,
//                                    resource: &ResourceAddress) -> Decimal
//{
//    let change =
//        result.vault_balance_changes()
//        .get(&GlobalAddress::from(*component)).unwrap()
//        .get(resource).unwrap();
//    match change {
//        BalanceChange::Fungible(amount) => *amount,
//        BalanceChange::NonFungible { added, removed } =>
//            (added.len() as i64 - removed.len() as i64).into(),
//    }
//}

//pub fn get_component_non_fungible_balance_change(
//    result: &CommitResult,
//    component: &ComponentAddress,
//    resource: &ResourceAddress) ->
//    (BTreeSet<NonFungibleLocalId>, BTreeSet<NonFungibleLocalId>)
//{
//    let change =
//        result.balance_changes()
//        .get(&GlobalAddress::from(*component)).unwrap()
//        .get(resource).unwrap();
//    match change {
//        BalanceChange::Fungible(_) => panic!("resource fungible"),
//        BalanceChange::NonFungible { added, removed } =>
//            (added.clone(), removed.clone()),
//    }
//}


/// Creates the test runner, a user, and publishes the package under
/// test.
pub fn setup_for_test() -> (DefaultTestRunner, User, PackageAddress) {
    let mut test_runner =
        TestRunnerBuilder::new()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config()))
        .without_trace()
        .build();
    let alice = make_user(&mut test_runner, Some(&"Alice".to_owned()));
    let package_address = test_runner.compile_and_publish(this_package!());

    (test_runner, alice, package_address)
}


/// Gives a number of tokens from one user account to another. Works
/// for fungibles and non-fungibles alike although you cannot name
/// specific local ids to transfer.
pub fn give_tokens(test_runner: &mut DefaultTestRunner,
               giver_account: &ComponentAddress,
               giver_nfgid: &NonFungibleGlobalId,
               recip_account: &ComponentAddress,
               gift_token: &ResourceAddress,
               gift_amount: AskingType) {

    let amount: Decimal;

    let mut manifest = ManifestBuilder::new();

    match gift_amount {
        AskingType::Fungible(q) => amount = q,
        AskingType::NonFungible(nflids, q) => {
            amount = q.unwrap_or_default().into();
            if let Some(nflids) = nflids {
                manifest = manifest
                    .withdraw_non_fungibles_from_account(
                        *giver_account,
                        *gift_token,
                        nflids);
            }
        }
    }
    
    if !amount.is_zero() {
        manifest = manifest
        .withdraw_from_account(
            *giver_account,
            *gift_token,
            amount);
    }
    
    let manifest = manifest
        .try_deposit_batch_or_abort(*recip_account, ManifestExpression::EntireWorktop, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![giver_nfgid.clone()],
    );

    receipt.expect_commit_success();
}

use std::ops::Range;

/// Converts a vector of u64 into a vector of NonFungibleLocalId
pub fn to_nflids(ints: Range<u64>) -> BTreeSet<NonFungibleLocalId> {
    let ints: Vec<u64> = ints.collect();
    ints.into_iter().map(|n| NonFungibleLocalId::Integer(n.into())).collect()
}

/// Creates an NFT resource with integer-based local ids. Local ids
/// will start on `base` and count upwards until there are `amount`
/// NFTs in the resource. All NFTs will be given to `owner_account`.
pub fn create_nft_resource(test_runner: &mut DefaultTestRunner,
                           owner: &User,
                           base: u64,
                           amount: u64,
                           badge: Option<&ResourceAddress>) -> ResourceAddress {

    let owner_nfgid = NonFungibleGlobalId::from_public_key(&owner.pubkey);

    let roles = NonFungibleResourceRoles {
        mint_roles: mint_roles!(
            minter => if let Some(badge_resaddr) = badge {
                rule!(require(*badge_resaddr))
            } else {
                rule!(allow_all)
            };
            minter_updater => rule!(allow_all);
        ),
        withdraw_roles: withdraw_roles!(
            withdrawer => rule!(allow_all);
            withdrawer_updater => rule!(deny_all);
        ),
        deposit_roles: deposit_roles!(
            depositor => rule!(allow_all);
            depositor_updater => rule!(deny_all);
        ),
        burn_roles: None,
        freeze_roles: None,
        recall_roles: None,
        non_fungible_data_update_roles: None,
    };

    // We're just faking the simplest None we can get away with here
    // (because faking it with the usual None::<String> doesn't work
    // in this case)
    let empty_supply: Option<Vec<(NonFungibleLocalId, NfData)>> = None;
    let manifest = ManifestBuilder::new()
        .create_non_fungible_resource(
            OwnerRole::Fixed(rule!(require(owner_nfgid.clone()))),
            NonFungibleIdType::Integer,
            true,
            roles,
            metadata!(),
            empty_supply)
        .deposit_batch(owner.account)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        manifest,
        vec![owner_nfgid.clone()],
    );

    receipt.expect_commit_success();
    let resaddr =
        receipt
        .expect_commit(true)
        .new_resource_addresses()[0];

    let mut minted = 0;
    const BATCH_SIZE: u64 = 100;

    // We mint in batches because there is a max-substates-write limit
    // that we might hit otherwise when making lots of NFTs.
    while minted < amount {
        let mut to_mint = amount - minted;
        if to_mint > BATCH_SIZE { to_mint = BATCH_SIZE; }

        let mut builder = ManifestBuilder::new();
        if let Some(badge_resaddr) = badge {
            builder = builder.create_proof_from_account_of_amount(
                owner.account, *badge_resaddr, dec!(1));
        }
        let manifest =
            builder.mint_non_fungible(
                resaddr,
                (minted..minted+to_mint).map(
                    |n| (NonFungibleLocalId::Integer((base+n).into()),
                         NfData {}))
                    .collect::<HashMap<NonFungibleLocalId, NfData>>())
            .deposit_batch(owner.account)
            .build();
        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            vec![owner_nfgid.clone()],
        );

        receipt.expect_commit_success();
        minted += to_mint;
    }
    
    resaddr
}
