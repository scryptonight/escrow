//! A component that stores funds for you and lets you access them
//! yourself or issue Allowance NFTs to let others access them.
//!
//! Think of an Escrow as an account component external to your
//! wallet, which is nevertheless under your control. You can use this
//! Escrow as a central place to hold funds that go into a variety of
//! services.
//!
//! You can issue Allowance NFTs towards your Escrow and give those
//! Allowances to others. Each Allowance NFT acts a bit like an IOU
//! that the holder can use to extract funds from your Escrow, for as
//! long as funds last.
//!
//! You could for example put 10k XRD into an Escrow, issue an
//! Allowance for 10k XRD and give to your son, and another Allowance
//! also for 10k XRD and give to your daughter. You explain to them
//! that these are emergency funds they can dip into if they get into
//! an emergency situation while travelling. You don't expect these
//! funds to ever actually get used, but they are there in full for
//! each of your two kids should the emergency ever arise.
//!
//! A single Escrow component can hold any amount of token pools, each
//! associated with an owner badge. That is, Alice can have a token
//! pool where she stores all sorts of different tokens while Bob can
//! also have a token pool on the same Escrow component instance that
//! Alice does. And of course, Alice or Bob could even have multiple
//! such sets of pools tied to different ownership badges still on the
//! same Escrow instance. (In principle there is no reason the world
//! should need more than a single instance of this component.) For
//! this reason, when interacting with an Escrow instance you always
//! need to know which owner badge to access, and which resource type
//! you're interested in.
//!
//! # Some use cases
//!
//! ## Curbing your enthusiasm
//!
//! You could put funds into Escrow intended for playing an online
//! game and issue an Allowance to that game that lets it charge its
//! fees off of the Escrow. By disassociating these payments from your
//! main account you can feel more safe about giving the game direct
//! access to funds (since the Escrow only holds as much funds as you
//! put there), and the Allowance system gives you fine grained
//! control over how much and how often the game can charge you. Or
//! you can use it to curb your own excitement: if you set it up so
//! the game can at max extract, say, 100 XRD per month, then this
//! effectively caps how much you end up spending on the game when
//! otherwise the excitement of the moment might have you spending
//! hundreds in a week without even realizing what's going on before
//! it's too late.
//!
//! ## Central trading account
//!
//! You can use an Escrow as a central repository of funds that you
//! use for trading activities. Markets that recognize the Escrow
//! concept can make payouts for completed deals into the Escrow
//! (instead of e.g. holding it inside their own internal vaults until
//! you come to collect), and can pull funds for covering your trade
//! deals also from the same Escrow. You could set this up to be
//! self-perpetuating (if your trades are clever enough) in that the
//! funds that go into the Escrow from completed deals fund your
//! future deals that are being taken from the same Escrow. Then every
//! once in a while you can drop by and skim off some of the profits
//! that have been happening since last payday.
//!
//! ## Selling all your tokens on all the exchanges
//!
//! If you're selling a million tokens of some type then instead of
//! putting them all on a single exchange, and instead of having to
//! meticulously distribute them across multiple exchanges, you can
//! put your million tokens in an Escrow and put an Allowance for a
//! million tokens on every exchange in the world. *All* the exchanges
//! can then offer your million tokens for sale, and all of them will
//! be pulling from your Escrow to fulfill trades. Once the Escrow is
//! empty, the exchanges will automatically no longer be doing those
//! trades because pulling from the Escrow will yield nothing.
//!
//! Note that test scenario 1 demonstrates this type of use case.
//!
//! ## Offering to buy more than you can afford
//!
//! You could use an Escrow to set up more trade deals than you can
//! technically afford, by setting up Allowance-backed trade deals
//! that are only going to actually be completed for so long as the
//! Escrow holds the necessary funds. Maybe you have 10,000 XRD and
//! you want to buy either USD or EURO stablecoins depending on which
//! drops below a certain threshold first. You set up one limit buy
//! for 10k XRD worth of USD, and one limit buy for 10k XRD worth of
//! EURO, each backed by Allowances that can pull the full 10k XRD in
//! your Escrow. Whichever of those happens first will pull all your
//! funds (and then even if the other happens later the trade won't go
//! through because you'll be out of funds). Contrast this to having
//! to bind up XRD into each limit buy in which case you'd end up with
//! a 5k XRD limit buy of USD and a 5k XRD limit buy of EURO.
//!
//! Note that test scenario 2 demonstrates this type of use case.
//!
//! ## Giving someone a (traditional) allowance account
//!
//! You could set up an Escrow as a funding mechanism for someone
//! you're helping finance, putting funds into the Escrow and giving
//! them an Allowance to extract those funds. This would give you
//! control over their use of the funds (through Allowance rules)
//! without them having to involve you directly every single time they
//! need to make a withdrawal.
//!
//! # Subsidies
//!
//! You can subsidize (internally it uses `lock_fee` or
//! `lock_contingent_fee`) the current transaction by pulling funds
//! from your Escrow, using the [subsidize] and [subsidize_contingent]
//! functions.
//!
//! You can also allow others to subsidize transactions by giving them
//! an Allowance and having them use [subsidize_with_allowance] to
//! pull XRD from your Escrow to run their transactions. You might be
//! doing this as a marketing stunt, or it may be a customer support
//! service, or perhaps it's to let a service run transactions on your
//! behalf etc.
//!
//! When using [subsidize_with_allowance] note the following:
//!
//! - If you're using an Accumulating allowance, that allowance will
//! be reduced by the amount you specify in the function call
//! regardless of how much actually gets spent on the subsidy. This is
//! because the component cannot know how much in fees you actually
//! end up paying. For example if you specify max 10 XRD in the
//! function call and only 5 XRD gets spent, the allowance remaining
//! amount is nevertheless reduced by 10 XRD.
//!
//! - Failed subsidized transactions will pull funds from your escrow
//! pool but this will not be reflected in the Allowance itself since
//! failed transactions cause no on-ledger changes apart from tx
//! cost. This means that a malicious allowance holder for XRD can use
//! it to drain your XRD by spamming failing transactions. For this
//! reason only issue XRD allowances to parties you can trust to be
//! honest. (This problem could be mitigated by introducing
//! configuration parameters to allow/disallow subsidies but the
//! current implementation is more intended to be a tech demo than a
//! fully deployable product and so this has not been done.)
//!
//! # Automatic Allowance issuance
//!
//! You can maintain a list of trusted parties who will automatically
//! receive Allowances whenever they deposit funds into your
//! account. You may use this if you have multiple different services
//! interacting with your Escrow pool (different market places
//! perhaps) and you want each such service to be able to take back
//! out funds they put in, in order to spend them on services or
//! trades.
//!
//! Such a party will need to provide a badge with a trusted
//! non-fungible global id, or simply with a trusted resource address,
//! when calling [deposit_funds] and they will receive back an
//! Allowance for that amount of the deposit.
//!
//! # Public API
//!
//! - [instantiate_escrow] Create a new Escrow instance.
//!
//! - [deposit_funds] Add funds to your Escrow pool.
//!
//! - [read_funds] See how many funds you have available.
//!
//! - [withdraw] Pull funds out of your Escrow pool.
//!
//! - [withdraw_with_allowance] Use an Allowance to pull funds out of
//! someone's Escrow pool.
//!
//! - [withdraw_all_of] Pull all funds of one type out of your Escrow
//! pool.
//!
//! - [subsidize] Pull XRD from your Escrow pool to subsidize the
//! current transaction.
//!
//! - [subsidize_with_allowance] Use an Allowance to pull XRD out of
//! someone's Escrow pool to subsidize the current transaction.
//!
//! - [subsidize_contingent] Pull XRD from your Escrow pool to
//! subsidize (contingent on its success) the current transaction.
//!
//! - [mint_allowance] Create an Allowance for your Escrow pool.
//!
//! - [reduce_allowance_to_amount] Reduce the amount of funds
//! available in an Allowance you control.
//!
//! - [reduce_allowance_by_nflids] Reduce the set of non-fungible ids
//! available in an Allowance you control.
//!
//! - [add_trusted_nfgid] Add a non-fungible global id to the list of
//! badges you trust to be given automatic Allowances when depositing
//! to your Escrow pool.
//!
//! - [remove_trusted_nfgid] Remove a non-fungible id to the list of
//! badges you trust.
//!
//! - [is_nfgid_trusted] Checks if a given non-fungible is trusted by
//! a specific Escrow pool.
//!
//! - [add_trusted_resource] Add a resource, all badges of which you
//! trust to be given automatic Allowances.
//!
//! - [remove_trusted_resource] Remove a resource from the trusted
//! resource list.
//!
//! - [is_resource_trusted] Checks if a given resource is trusted.
//!
//!
//! # About error messages
//!
//! Many error messages that are produced on panics in this code are
//! preceded with an error code, starting at error code 2000 for this
//! project. They are there to make each error easily recognizable in
//! the test suite so that I can easily test that a transaction that
//! is intended to fail fails for the correct reason.
//!
//! Note that as this is a fairly new idea, not all the error messages
//! have codes at this point.
//!
//! # About IndexSet sizes for non-fungible local ids
//!
//! Note that use of TokenQuantity introduces a possible cost unit
//! limit problem when the IndexSet inside gets large. The current
//! implementation is suitable for a moderate number of
//! nonfungible-ids in that set, but not very many.
//!
//! [instantiate_escrow]: crate::escrow::Escrow::instantiate_escrow
//! [deposit_funds]: crate::escrow::Escrow::deposit_funds
//! [read_funds]: crate::escrow::Escrow::read_funds
//! [withdraw]: crate::escrow::Escrow::withdraw
//! [withdraw_with_allowance]: crate::escrow::Escrow::withdraw_with_allowance
//! [withdraw_all_of]: crate::escrow::Escrow::withdraw_all_of
//! [subsidize]: crate::escrow::Escrow::subsidize
//! [subsidize_with_allowance]: crate::escrow::Escrow::subsidize_with_allowance
//! [subsidize_contingent]: crate::escrow::Escrow::subsidize_contingent
//! [mint_allowance]: crate::escrow::Escrow::mint_allowance
//! [reduce_allowance_to_amount]: crate::escrow::Escrow::reduce_allowance_to_amount
//! [reduce_allowance_by_nflids]: crate::escrow::Escrow::reduce_allowance_by_nflids
//! [add_trusted_nfgid]: crate::escrow::Escrow::add_trusted_nfgid
//! [remove_trusted_nfgid]: crate::escrow::Escrow::remove_trusted_nfgid
//! [is_nfgid_trusted]: crate::escrow::Escrow::is_nfgid_trusted
//! [add_trusted_resource]: crate::escrow::Escrow::add_trusted_resource
//! [remove_trusted_resource]: crate::escrow::Escrow::remove_trusted_resource
//! [is_resource_trusted]: crate::escrow::Escrow::is_resource_trusted

use scrypto::prelude::*;

pub mod util;
pub mod token_quantity;
pub mod mock_dex;

use radix_engine_common::ManifestSbor;
use util::{unix_time_now, length_of_option_set, unchecked_proof_to_nfgid};
use token_quantity::TokenQuantity;

/// Defines the lifecycle of an Allowance NFT. Every Allowance follows
/// exactly one of these strategies.
#[derive(ScryptoSbor, ManifestSbor)]
pub enum AllowanceLifeCycle {
    /// Allowance NFT will be burnt after first use.
    OneOff,

    /// Each use reduces `max_amount`. Allowance NFT will be burnt
    /// when `max_amount` reaches 0. (If `max_amount` is `None` then
    /// this is effectively the same as a `OneOff` lifecycle.)
    Accumulating,

    /// The allowance can be used any number of times, each time for
    /// up to `max_amount` tokens. It cannot be used more frequently
    /// than once every `min_delay` seconds. If this is set to zero it
    /// will still prevent multiple uses within the same "second".
    ///
    /// Note that if `min_delay` is set to `None` the allowance can be
    /// used multiple times per "second" and even multiple times in
    /// the same transaction manifest.
    ///
    /// Also note that the ledger has a maximum clock accuracy which
    /// means that the preceding "per second" may actually be per
    /// minute.
    ///
    /// A `Repeating` allowance NFT will never be burnt by this
    /// component.
    Repeating{min_delay: Option<i64>},
}

/// An Allowance NFT allows its owner to extract some amount of tokens
/// from a pool. It is issued by the pool's owner, or it can be
/// returned to a depositor that is trusted by the pool owner.
///
/// Note that in the current implementation the owner of the pool is
/// able to recall any Allowance NFT issued for that pool. This may be
/// undesirable for some use cases, e.g. if you're selling an
/// Allowance to someone then they may want to have guarantees that it
/// won't be arbitrarily recalled before they use it. This component
/// should be fairly easy to modify to cover such use cases, e.g. by
/// having a configuration option or function parameter controlled by
/// the owner that allows the creation of non-recallable Allowance
/// NFTs.
#[derive(ScryptoSbor, NonFungibleData)]
pub struct AllowanceNfData {
    /// The Escrow pool this allowance is associated with.
    pub escrow_pool: (ComponentAddress, NonFungibleGlobalId),
    
    /// If set, this is the latest Unix time at which the allowance can
    /// be used.
    pub valid_until: Option<i64>,

    /// The allowance cannot be used until this Unix time. Set to zero
    /// if you don't want to use this.
    ///
    /// In addition to determining the first possible use time, this
    /// field is also used for tracking the next allowed use if the
    /// allowance type is `Repeating` with a delay set.
    #[mutable]
    pub valid_from: i64,

    /// How to deal with this allowance being used multiple times.
    pub life_cycle: AllowanceLifeCycle,

    /// The resource this allowance is for.
    pub for_resource: ResourceAddress,

    /// The amount of this resource that can be taken. If this is
    /// `None` then any amount can be taken.
    #[mutable]
    pub max_amount: Option<TokenQuantity>,
}

impl AllowanceNfData {
    /// Checks our time-based validity.
    pub fn is_valid(&self) -> bool {
        let now = unix_time_now();
        self.valid_from >= now &&
            (self.valid_until.is_none()
            || self.valid_until.unwrap() < now)
    }
}

/// A pool holds any number of vaults in it, one vault for each type
/// of resource that has ever been in the pool. Each pool has its own
/// resource address for Allowance NFTs, and a set of trusted badges
/// and badge resources.
#[derive(ScryptoSbor)]
struct Pool {
    /// Our Allowances are of this non-fungible resource.
    allowance_badge_res: ResourceAddress,
    /// We trust these badges enough to give them back Allowances to
    /// cover any deposits they make with us.
    trusted_nfgids: KeyValueStore<NonFungibleGlobalId, bool>,
    /// We trust holders of badges of these resources enough to give
    /// them back Allowances to cover any deposits they make with us.
    trusted_res: KeyValueStore<ResourceAddress, bool>,
    /// Contains all the pool funds.
    vaults: KeyValueStore<ResourceAddress, Vault>,
}

#[blueprint]
mod escrow {
    use crate::AllowanceLifeCycle;

    /// An Escrow is just a bunch of pools, each pool tied to the
    /// badge of its owner.
    struct Escrow {
        pools: KeyValueStore<NonFungibleGlobalId, Pool>,
    }

    impl Escrow {
        /// Note that the owner of an Escrow component has no
        /// particular power over it or its users.
        pub fn instantiate_escrow() -> Global<Escrow>
        {
            Self {
                pools: KeyValueStore::new(),
            }
            .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        /// Anyone can deposit funds into any escrow pool.
        ///
        /// The `funds` are deposited into the pool owned by `owner`.
        ///
        /// If the depositor provides a proof `allowance_requestor`
        /// showing that they are a trusted agent they will receive
        /// back an Allowance to withdraw the amount that they
        /// deposited in this call.
        pub fn deposit_funds(&mut self,
                             owner: NonFungibleGlobalId,
                             funds: Bucket,
                             allowance_requestor: Option<Proof>)
                             -> Option<Bucket>
        {
            let owner_nfgid = NonFungibleGlobalId::new(
                owner.resource_address(),
                owner.local_id().clone());

            {
                let pool = self.get_or_add_pool(&owner_nfgid);

                // Create this resource vault if we don't have it already.
                if pool.vaults.get(&funds.resource_address()).is_none() {
                    pool.vaults.insert(funds.resource_address(),
                                       Vault::new(funds.resource_address()));
                }
            }

            // Create allowance if requested and allowed
            let maybe_allowance_bucket =
                if allowance_requestor.is_none() { None } else
            {
                let requestor = allowance_requestor.unwrap().skip_checking();
                assert!(self.is_resource_trusted(owner.clone(),
                                                 requestor.resource_address())
                        || self.is_nfgid_trusted(
                            owner.clone(),
                            NonFungibleGlobalId::new(
                                requestor.resource_address(),
                                requestor.as_non_fungible().non_fungible_local_id())),
                        "2013 only trusted can request allowance");
                let max_amount =
                    if funds.resource_address().is_fungible() {
                        TokenQuantity::Fungible(funds.amount())
                    } else {
                        TokenQuantity::NonFungible(
                            Some(funds.as_non_fungible().non_fungible_local_ids()),
                            None)
                    };
                let pool_mgr = ResourceManager::from(
                    self.get_or_add_pool(&owner_nfgid).allowance_badge_res);
                Some(self.create_allowance(
                    (Runtime::global_address(), owner),
                    pool_mgr,
                    None,
                    0,
                    AllowanceLifeCycle::Accumulating,
                    funds.resource_address(),
                    Some(max_amount)))
            };
            
            // Pool the funds
            let mut pool = self.get_or_add_pool(&owner_nfgid);
            pool.vaults.get_mut(&funds.resource_address()).unwrap().put(funds);

            maybe_allowance_bucket
        }

        /// Returns the amount of tokens available for the named
        /// resource in the named pool. If the pool doesn't exist or
        /// doesn't have that resource we return zero.
        pub fn read_funds(&self,
                          owner: NonFungibleGlobalId,
                          resource: ResourceAddress) -> Decimal
        {
            let pool = self.pools.get(&owner);
            if let Some(pool) = pool {
                let vault = pool.vaults.get(&resource);
                if let Some(vault) = vault {
                    return vault.amount()
                }
            }
            Decimal::ZERO
        }

        /// The pool owner can use this function to withdraw funds
        /// from their pool. `caller` must be a proof of the pool
        /// owner.
        ///
        /// If the requested tokens aren't available we will panic.
        pub fn withdraw(&mut self,
                        caller: Proof,
                        resource: ResourceAddress,
                        quantity: TokenQuantity) -> Bucket
        {
            let (take_nflids, take_amount) = quantity.extract_max_values();
            self.operate_on_vault(
                &unchecked_proof_to_nfgid(caller),
                &resource,
                None,
                |mut v| {
                    let mut bucket: Option<Bucket> = None;

                    // First take the named nflids: if we do this the
                    // other way around they may no longer be
                    // available when we try.
                    if let Some(nflids) = &take_nflids {
                        bucket = Some(v.as_non_fungible().take_non_fungibles(&nflids).into());
                    }
                    // Then take the necessary amount of arbitrary tokens.
                    if let Some(amount) = take_amount {
                        if let Some(ref mut bucket) = bucket {
                            bucket.put(v.take(amount));
                        } else {
                            bucket = Some(v.take(amount));
                        }
                    }
                    bucket
                })
                .unwrap()
        }

        /// Someone who (typically) isn't the owner of a pool can
        /// withdraw from it if they can provide an appropriate
        /// Allowance NFT supporting that withdrawal.
        ///
        /// The `allowance` is supplied in a Bucket so that we can
        /// burn it if it's now spent. For this reason it may or may
        /// not be returned back to the caller, and its authorization
        /// details may have changed if it *is* returned.
        pub fn withdraw_with_allowance(&mut self,
                                       allowance: Bucket,
                                       quantity: TokenQuantity)
                                       -> (Bucket, Option<Bucket>)
        {
            let allowance_resaddr = allowance.resource_address();
            let (owner_nfgid, withdraw_resource, allowance) =
                self.use_allowance(allowance, quantity.clone());

            // Note the allowance NFT may have been burned by this
            // point

            let (take_nflids, take_amount) = quantity.extract_max_values();
            
            (self.operate_on_vault(
                &owner_nfgid,
                &withdraw_resource,
                Some(allowance_resaddr),
                |mut v| {
                    let mut bucket: Option<Bucket> = None;

                    // First take the named nflids: if we do this the
                    // other way around they may no longer be
                    // available when we try.
                    if let Some(nflids) = &take_nflids {
                        bucket = Some(v.as_non_fungible().take_non_fungibles(&nflids).into());
                    }
                    // Then take the necessary amount of arbitrary tokens.
                    if let Some(amount) = take_amount {
                        if let Some(ref mut bucket) = bucket {
                            bucket.put(v.take(amount));
                        } else {
                            bucket = Some(v.take(amount));
                        }
                    }
                    bucket
                })
             .unwrap(),
             allowance)
        }

        /// The owner of a pool can call this convenience function to
        /// withdraw all tokens of a given resource. Note that this
        /// may panic on exceeding cost unit limits if you're trying
        /// withdraw a large number of non-fungible tokens. In this
        /// case do a more controlled withdrawal of smaller sets of
        /// non-fungibles instead.
        ///
        /// The `caller` must be a proof of the pool owner.
        pub fn withdraw_all_of(&mut self,
                               caller: Proof,
                               resource: ResourceAddress)
                               -> Bucket
        {
            self.operate_on_vault(&unchecked_proof_to_nfgid(caller),
                                  &resource,
                                  None,
                                  |mut v| Some(v.take_all()))
                .unwrap()
        }

        /// The owner of a pool can pull XRD from their pool to lock
        /// fees for the current transaction.
        ///
        /// The `caller` must be a proof of the pool owner.
        pub fn subsidize(&mut self,
                         caller: Proof,
                         amount: Decimal)
        {
            self.operate_on_vault(&unchecked_proof_to_nfgid(caller),
                                  &XRD,
                                  None,
                                  |v| {v.as_fungible().lock_fee(amount); None});
        }

        /// A holder of an Allowance for XRD can pull XRD from the
        /// pool to lock fees for the current transaction.
        ///
        /// The `allowance` bucket must contain an Allowance for
        /// sufficient XRD.
        ///
        /// Note that the Allowance may get burned during execution
        /// and if so will not be returned to the caller.
        pub fn subsidize_with_allowance(&mut self,
                         allowance: Bucket,
                         amount: Decimal) -> Option<Bucket>
        {
            let allowance_resaddr = allowance.resource_address();
            let (owner_nfgid, withdraw_resource, allowance) =
                self.use_allowance(allowance, TokenQuantity::Fungible(amount));

            // Note the allowance NFT may have been burned by this
            // point

            assert_eq!(XRD, withdraw_resource,
                       "only XRD can by used for subsidy");
                
            self.operate_on_vault(&owner_nfgid,
                                  &XRD,
                                  Some(allowance_resaddr),
                                  |v| {v.as_fungible().lock_fee(amount); None});

            allowance
        }

        /// The owner of a pool can pull XRD from their pool to lock
        /// contingent fees for the current transaction.
        ///
        /// The `caller` must be a proof of the pool owner.
        pub fn subsidize_contingent(&mut self,
                                    caller: Proof,
                                    amount: Decimal)
        {
            self.operate_on_vault(
                &unchecked_proof_to_nfgid(caller),
                &XRD,
                None,
                |v| {v.as_fungible().lock_contingent_fee(amount); None});
        }

        /// The owner of a pool can mint allowances for that pool, and
        /// can then distribute those allowance NFTs to whoever.
        ///
        /// The allowance will be for the pool owned by the `owner`
        /// proof. If a pool doesn't yet exist, one will be created.
        ///
        /// Otherwise, specify the parameters of the allowance (see
        /// the doc for the AllowanceNfData struct for details), and
        /// the newly created allowance will be returned out of this
        /// function.
        pub fn mint_allowance(&mut self,
                              owner: Proof,
                              valid_until: Option<i64>,
                              valid_from: i64,
                              life_cycle: AllowanceLifeCycle,
                              for_resource: ResourceAddress,
                              max_quantity: Option<TokenQuantity>) -> Bucket
        {
            if let Some(max_quantity) = &max_quantity {
                let amount;
                match max_quantity {
                    TokenQuantity::NonFungible(_, max_amount) => {
                        amount = max_amount.map(|v|Decimal::from(v))
                    },
                    TokenQuantity::Fungible(max_amount) => {
                        amount = Some(*max_amount)
                    },
                }
                assert!(!amount.unwrap_or_default().is_negative(),
                        "max_amount cannot be negative");
            }

            // Access control is effectively enforced through our pool
            // lookup further down.
            let owner = unchecked_proof_to_nfgid(owner);

            let pool_mgr = ResourceManager::from(
                self.get_or_add_pool(&owner).allowance_badge_res);

            self.create_allowance(
                (Runtime::global_address(), owner),
                pool_mgr,
                valid_until,
                valid_from,
                life_cycle,
                for_resource,
                max_quantity)
        }

        /// Anyone who holds an `allowance` NFT can voluntarily reduce
        /// its max amount by calling this function. Just provide a
        /// proof that you control the allowance and you're good.
        ///
        /// Make sure that `new_max` is lower than (or equal to) the
        /// `max_amount` currently in the allowance or this function
        /// will panic. Also don't send in a negative number.
        ///
        /// On a NonFungible type allowance this only reduces the
        /// numerical amount, never the set of nflids available if
        /// any.
        pub fn reduce_allowance_to_amount(&self,
                                          allowance: Proof,
                                          new_max: Decimal)
        {
            assert!(!new_max.is_negative(),
                    "2003 allowance can't be negative");
            
            // Access control is effectively achieved through the
            // use of the proof's resource address and nflid later.
            let allowance = allowance.skip_checking();

            let nfdata: AllowanceNfData =
                ResourceManager::from(allowance.resource_address())
                .get_non_fungible_data(&allowance.as_non_fungible().non_fungible_local_id());

            let new_token_quantity: TokenQuantity;

            if let Some(max_amount) = nfdata.max_amount {
                match max_amount {
                    TokenQuantity::Fungible(amount) => {
                        assert!(amount >= new_max, "2000 allowance increase not allowed");
                        new_token_quantity = TokenQuantity::Fungible(new_max);
                    },
                    TokenQuantity::NonFungible(nflids, amount) => {
                        if let Some(amount) = amount {
                            assert!(new_max < amount.into(),
                                    "2001 allowance increase not allowed");
                            new_token_quantity = TokenQuantity::NonFungible(
                                nflids,
                                Some(u64::try_from(new_max).expect(
                                    "2004 new_max must be a whole number")));
                        } else {
                            panic!("2002 allowance increase not allowed");
                        }
                    },
                }
            } else {
                // Since `None` means infinite, changing from `None`
                // to any number is a reduction and therefore allowed.
                new_token_quantity = TokenQuantity::Fungible(new_max);
            }

            ResourceManager::from(allowance.resource_address())
                .update_non_fungible_data(&allowance.as_non_fungible().non_fungible_local_id(),
                                          "max_amount",
                                          Some(new_token_quantity));
        }

        /// Anyone who holds an `allowance` NFT of NonFungible type
        /// can voluntarily reduce its availeble nflids by calling
        /// this function. Just provide a proof that you control the
        /// allowance and you're good.
        ///
        /// Any nflids you specify in `to_remove` will be removed from
        /// your list of available ones. If you specify more nflids
        /// than are actually in the Allowance those nflids are just
        /// ignored in our processing of them.
        ///
        /// On a NonFungible type allowance this only reduces the
        /// nflids set, never the additional fixed amount.
        ///
        /// This function will panic if you try to call it on a
        /// Fungible allowance, or if your allowance doesn't currently
        /// have an nflids set defined.
        pub fn reduce_allowance_by_nflids(&self,
                                          allowance: Proof,
                                          to_remove: IndexSet<NonFungibleLocalId>)
        {
            // Access control is effectively achieved through the use
            // of the proof's resource address and nflid when
            // retrieving nfdata below.
            let allowance = allowance.skip_checking();

            let nfdata: AllowanceNfData =
                ResourceManager::from(allowance.resource_address())
                .get_non_fungible_data(&allowance.as_non_fungible().non_fungible_local_id());

            let new_token_quantity: TokenQuantity;

            if let Some(max_amount) = nfdata.max_amount {
                match max_amount {
                    TokenQuantity::Fungible(..) => {
                        panic!("2005 use reduce_allowance_to_amount for Fungible allowance");
                    },
                    TokenQuantity::NonFungible(nflids, amount) => {
                        if let Some(nflids) = nflids {
                            let new_nflids = Some(nflids.difference(&to_remove).cloned().collect());
                            new_token_quantity = TokenQuantity::NonFungible(
                                new_nflids,
                                amount);
                        } else {
                            panic!("2006 no nflids to remove from");
                        }
                    },
                }
            } else {
                panic!("2007 no nflids to remove from");
            }

            ResourceManager::from(allowance.resource_address())
                .update_non_fungible_data(&allowance.as_non_fungible().non_fungible_local_id(),
                                          "max_amount",
                                          Some(new_token_quantity));
        }

        /// The pool owner can add a non-fungible global id that is
        /// trusted to receive automatically generated Allowances when
        /// calling deposit_funds.
        pub fn add_trusted_nfgid(&mut self,
                                 owner: Proof,
                                 add_nfgid: NonFungibleGlobalId)
        {
            let owner_nfgid = unchecked_proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_nfgids.insert(add_nfgid, true);
        }

        /// The pool owner can remove a non-fungible global id from
        /// those trusted to receive automatically generated
        /// Allowances when calling deposit_funds.
        pub fn remove_trusted_nfgid(&mut self,
                                    owner: Proof,
                                    remove_nfgid: NonFungibleGlobalId)
        {
            let owner_nfgid = unchecked_proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_nfgids.insert(remove_nfgid, false);
        }

        /// Determines if a given non-fungible global id is currently
        /// trusted to receive automatically generated Allowances when
        /// calling deposit_funds.
        pub fn is_nfgid_trusted(&self,
                                owner: NonFungibleGlobalId,
                                candidate: NonFungibleGlobalId) -> bool
        {
            if let Some(pool) = self.pools.get(&owner) {
                self.is_nfgid_trusted_by_pool(&pool, candidate)
            } else {
                false
            }
        }

        /// The pool owner can add an entire resource, all tokens of
        /// which are trusted to receive automatically generated
        /// Allowances when calling deposit_funds.
        ///
        /// Note that if you add a fungible resource as trusted, trust
        /// can be established by presenting proof for a very tiny
        /// amount of tokens - down to the finest resolution of that
        /// resource (e.g. 1e-18 tokens). If you want *everyone* to be
        /// trusted then add XRD as a trusted resource and anyone can
        /// present a valid Proof if they want to.
        pub fn add_trusted_resource(&mut self,
                                    owner: Proof,
                                    add_resource: ResourceAddress)
        {
            let owner_nfgid = unchecked_proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_res.insert(add_resource, true);
        }

        /// The pool owner can remove a resource from those which are
        /// trusted to receive automatically generated Allowances when
        /// calling deposit_funds.
        pub fn remove_trusted_resource(&mut self,
                                       owner: Proof,
                                       remove_resource: ResourceAddress)
        {
            let owner_nfgid = unchecked_proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_res.insert(remove_resource, false);
        }
        
        /// Determines if a given resource is currently trusted to
        /// receive automatically generated Allowances when calling
        /// deposit_funds.
        pub fn is_resource_trusted(&self,
                                   owner: NonFungibleGlobalId,
                                   candidate: ResourceAddress) -> bool
        {
            if let Some(pool) = self.pools.get(&owner) {
                self.is_resource_trusted_by_pool(pool, candidate)
            } else {
                false
            }
        }

        //
        // Internal helper functions follow
        //
        
        /// Expends an allowance (or part of it) by checking that it
        /// is currently able to support the withdrawal indicated, and
        /// updating the allowance to reflect such a withdrawal having
        /// taken place. This potentially burns the allowance so after
        /// calling this function it could be gone.
        ///
        /// This function does not do the actual withdrawing, the
        /// caller has to see to that.
        ///
        /// NOTE we do *not* check that the allowance has the correct
        /// resource address for the pool as we have no information on
        /// the pool. This must have been already checked by the
        /// calling party.
        fn use_allowance(&self, allowance: Bucket, amount: TokenQuantity)
                         -> (NonFungibleGlobalId, ResourceAddress, Option<Bucket>)
        {
            let nfdata: AllowanceNfData =
                ResourceManager::from(allowance.resource_address())
                .get_non_fungible_data(
                    &allowance.as_non_fungible().non_fungible_local_id());
            let mut burn = false;
            let owner_nfgid = nfdata.escrow_pool.1;
            let withdraw_resource = nfdata.for_resource;

            let (take_nflids, take_amount) = amount.extract_max_values();

            assert_eq!(Runtime::global_address(), nfdata.escrow_pool.0,
                       "allowance is not for this escrow");

            let now = unix_time_now();
            assert!(nfdata.valid_from <= now,
                    "2009 allowance not yet valid");
            assert!(nfdata.valid_until.is_none()
                    || nfdata.valid_until.unwrap() >= now,
                    "2011 allowance no longer valid");

            // Check that the allowance is big enough for this
            // withdrawal.
            match nfdata.max_amount {
                Some(TokenQuantity::NonFungible(ref max_nflids, max_amount)) => {
                    let mut already_taken = 0;
                    if let Some(take_nflids) = &take_nflids {
                        if let Some(max_nflids) = max_nflids {
                            // The NFTs we can take out of max_nflids
                            // shouldn't count towards the max_amount
                            // further down.
                            already_taken = take_nflids.intersection(max_nflids).count();
                        }
                    }
                    let to_take = take_amount.unwrap_or_default()
                        + length_of_option_set(&take_nflids) - already_taken;
                    assert!(to_take <= max_amount.unwrap_or_default().into(),
                            "2012 insufficient allowance");
                },
                Some(TokenQuantity::Fungible(max_amount)) => {
                    assert!(take_amount.unwrap_or_default() + length_of_option_set(&take_nflids)
                            <= max_amount,
                            "2010 insufficient allowance");
                },
                None => {
                    // This means there is no limit
                },
            }

            // Update the allowance to reflect the withdrawal
            // indicated.
            match nfdata.life_cycle {
                AllowanceLifeCycle::OneOff => {
                    burn = true;
                },
                AllowanceLifeCycle::Accumulating => {
                    match nfdata.max_amount {
                        Some(TokenQuantity::Fungible(max_amount)) => {
                            let new_max_amount = max_amount
                                - take_amount.unwrap_or_default()
                                - length_of_option_set(&take_nflids);
                            if new_max_amount.is_zero() { burn = true } else {
                                ResourceManager::from(allowance.resource_address())
                                    .update_non_fungible_data(
                                        &allowance.as_non_fungible().non_fungible_local_id(),
                                        "max_amount",
                                        Some(TokenQuantity::Fungible(new_max_amount)));
                            }
                        },
                        Some(TokenQuantity::NonFungible(max_nflids, max_amount)) => {
                            let new_max_nflids: Option<IndexSet<NonFungibleLocalId>>;
                            if let Some(take_nflids) = take_nflids {
                                new_max_nflids = Some(
                                    max_nflids.unwrap().difference(&take_nflids).cloned().collect());
                            } else {
                                new_max_nflids = max_nflids;
                            }
                            let new_max_amount: Option<u64>;
                            if let Some(take_amount) = take_amount {
                                new_max_amount = Some(max_amount.unwrap() - u64::try_from(take_amount)
                                                  .expect("amount to take must be whole number"));
                            } else {
                                new_max_amount = max_amount;
                            }
                            if new_max_amount.unwrap_or_default() == 0 && length_of_option_set(&new_max_nflids) == 0 {
                                // No tokens left in allowance
                                burn = true;
                            } else {
                                ResourceManager::from(allowance.resource_address())
                                    .update_non_fungible_data(
                                        &allowance.as_non_fungible().non_fungible_local_id(),
                                        "max_amount",
                                        Some(TokenQuantity::NonFungible(new_max_nflids, new_max_amount)));
                            }
                            
                        },
                        None => {
                            burn = true;
                        },
                    }
                },
                AllowanceLifeCycle::Repeating{min_delay} => {
                    if let Some(min_delay) = min_delay {
                        ResourceManager::from(allowance.resource_address())
                            .update_non_fungible_data(
                                &allowance.as_non_fungible().non_fungible_local_id(),
                                "valid_from",
                                now + min_delay);
                    }
                }
            }

            // Burn the allowance NFT if it's now spent
            let mut allowance = Some(allowance);
            if burn {
                allowance.unwrap().burn();
                allowance = None;
            }
            
            (owner_nfgid, withdraw_resource, allowance)
        }

        /// Determines if this nfgid is trusted to receive automatic
        /// Allowances. Does not consider whether its *resource* is
        /// trusted, check that individually.
        fn is_nfgid_trusted_by_pool(&self,
                                    pool: &KeyValueEntryRef<Pool>,
                                    candidate: NonFungibleGlobalId) -> bool
        {
            if let Some(trust) = pool.trusted_nfgids.get(&candidate) {
                return *trust
            }
            false
        }

        /// Determines if this resource is trusted to receive
        /// automatic Allowances.
        fn is_resource_trusted_by_pool(&self,
                                       pool: KeyValueEntryRef<Pool>,
                                       candidate: ResourceAddress) -> bool
        {
            if let Some(trust) = pool.trusted_res.get(&candidate) {
                *trust
            } else {
                false
            }
        }

        /// Allows you to do operations on a given pool vault, running
        /// `operation` and passing the vault corresponding to the
        /// requested `resource` to it.
        ///
        /// NOTE when calling this function, `owner` must be an
        /// already authorized identity as it is used to grant access
        /// to vaults. It *must* have come out of a proof, or out of a
        /// bucket, or otherwise from a trusted source.
        ///
        /// If 'allowance_resaddr' is present then authority to
        /// operate on the vault came from an allowance NFT with that
        /// resource address. This function asserts that that is
        /// indeed the resource address of the pool's allowance NFTs.
        fn operate_on_vault<F>(&mut self,
                               owner: &NonFungibleGlobalId,
                               resource: &ResourceAddress,
                               allowance_resaddr: Option<ResourceAddress>,
                               operation: F) 
                               -> Option<Bucket>
        where F: Fn(KeyValueEntryRefMut<Vault>) -> Option<Bucket>
        {
            if let Some(mut pool) = self.pools.get_mut(&owner) {
                if let Some(allowance_resaddr) = allowance_resaddr {
                    assert_eq!(pool.allowance_badge_res,
                               allowance_resaddr,
                               "allowance is not for this pool");
                }
                if let Some(vault) = pool.vaults.get_mut(&resource) {
                    operation(vault)
                } else {
                    panic!("resource not found")
                }
            } else {
                panic!("pool not found")
            }
        }

        /// Retrieves the pool corresponding to the input
        /// `owner_nfgid`, creating one if necessary.
        fn get_or_add_pool(&mut self,
                           owner_nfgid: &NonFungibleGlobalId)
                           -> KeyValueEntryRefMut<Pool>
        {
            // Create this pool if we don't have it already
            if self.pools.get(owner_nfgid).is_none() {

                // Each pool has its own allowance NF resource.
                let allowance_badge_mgr =
                    ResourceBuilder::new_ruid_non_fungible::<AllowanceNfData>(
                        OwnerRole::None)
                    .metadata(metadata!(init {
                        "name" => "Escrow allowance", locked; }))

                // All minting is done by the Escrow component, with
                // access check being situational.
                    .mint_roles(mint_roles!(
                        minter => rule!(require(global_caller(Runtime::global_address())));
                        minter_updater => rule!(deny_all);))

                // The escrow pool owner can recall any allowances
                // that have been issued for the pool.
                    .recall_roles(recall_roles!(
                        recaller => rule!(require(owner_nfgid.clone()));
                        recaller_updater => rule!(deny_all);
                    ))

                // Anyone can burn the allowances they have been
                // given.
                    .burn_roles(burn_roles!(
                        burner => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    ))

                // All nfdata manipulation is done by the Escrow
                // component.
                    .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                        non_fungible_data_updater =>
                            rule!(require(global_caller(Runtime::global_address())));
                        non_fungible_data_updater_updater => rule!(deny_all);
                    ))
                    
                    .create_with_no_initial_supply();
                
                self.pools.insert(
                    owner_nfgid.clone(),
                    Pool {
                        allowance_badge_res: allowance_badge_mgr.address(),
                        trusted_nfgids: KeyValueStore::new(),
                        trusted_res: KeyValueStore::new(),
                        vaults: KeyValueStore::new(),
                    });
            }

            self.pools.get_mut(owner_nfgid).unwrap()
        }

        /// Creates an Allowance NFT for `escrow_pool` using
        /// `pool_mgr` to do so, and initizalizing it with the given
        /// values.
        ///
        /// Note that all user authentication and authorization must
        /// have been done before calling this function.
        fn create_allowance(&self,
                            escrow_pool: (ComponentAddress, NonFungibleGlobalId),
                            pool_mgr: ResourceManager,
                            valid_until: Option<i64>,
                            valid_from: i64,
                            life_cycle: AllowanceLifeCycle,
                            for_resource: ResourceAddress,
                            max_amount: Option<TokenQuantity>) -> Bucket
        {
            pool_mgr
                .mint_ruid_non_fungible(
                    AllowanceNfData {
                        escrow_pool,
                        valid_until,
                        valid_from,
                        life_cycle,
                        for_resource,
                        max_amount
                    }
                )
        }
    }
}
