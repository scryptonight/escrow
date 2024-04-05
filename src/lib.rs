use scrypto::prelude::*;

pub mod util;
pub mod asking_type;

use radix_engine_common::ManifestSbor;
use util::unix_time_now;
use asking_type::AskingType;

#[derive(ScryptoSbor, ManifestSbor)]
pub enum AllowanceLifeCycle {
    /// Allowance NFT will be burnt after first use.
    OneOff,

    /// Each use reduces `max_amount`. Allowance NFT will be burnt
    /// when `max_amount` reaches 0.
    Accumulating,

    /// The allowance can be used any number of times, each time for
    /// up to `max_amount` tokens. It cannot be used more frequently
    /// than once every `min_delay` seconds. If set to zero it will
    /// still prevent multiple uses within the same "second".
    ///
    /// Note that if this is set to `None` the allowance can be used
    /// multiple times per "second" and even multiple times in the
    /// same transaction manifest.
    ///
    /// Also note that the ledger has a maximum clock accuracy that
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
#[derive(ScryptoSbor, NonFungibleData)]
pub struct AllowanceNfData {
    /// The Escrow pool this allowance is associated with.
    pub escrow_pool: (ComponentAddress, NonFungibleGlobalId),
    
    /// If set, this is the latest Unix time at which the allowance can
    /// be used.
    pub valid_until: Option<i64>,

    /// The allowance cannot be used until this Unix time is in the
    /// past. Set to zero if you don't want to use this.
    ///
    /// In addition to determining the first possible use time, this
    /// field is also used for tracking the next allowed use if the
    /// allowance type is `Repeating` with a delay set.
    #[mutable]
    pub valid_after: i64,

    /// How to deal with this allowance being used multiple times.
    pub life_cycle: AllowanceLifeCycle,

    /// The resource this allowance is for.
    pub for_resource: ResourceAddress,

    /// The amount of this resource that can be taken. Note that this
    /// is a meaningful limitation whether `for_resource` is fungible
    /// *or* non-fungible.
    #[mutable]
    pub max_amount: Option<Decimal>,

    // When Scrypto gives us a lazy-loading BTreeSet implementation we
    // may add this:
    // #[mutable]
    // allow_nflids: LazyBTreeSet<NonFungibleLocalId>,
}

#[derive(ScryptoSbor)]
struct Pool {
    allowance_badge_res: ResourceAddress,
    trusted_nfgids: KeyValueStore<NonFungibleGlobalId, bool>,
    trusted_res: KeyValueStore<ResourceAddress, bool>,
    vaults: KeyValueStore<ResourceAddress, Vault>,
}

#[blueprint]
mod escrow {
    use crate::AllowanceLifeCycle;

    struct Escrow {
        pools: KeyValueStore<NonFungibleGlobalId, Pool>,
    }

    impl Escrow {
        pub fn instantiate_escrow() -> Global<Escrow>
        {
            let addr =
                Self {
                    pools: KeyValueStore::new(),
                }
            .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();

            addr
        }

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
                        "only trusted can request allowance");
                let pool_mgr = ResourceManager::from(
                    self.get_or_add_pool(&owner_nfgid).allowance_badge_res);
                Some(self.create_allowance(
                    (Runtime::global_address(), owner),
                    pool_mgr,
                    None,
                    0,
                    AllowanceLifeCycle::Accumulating,
                    funds.resource_address(),
                    Some(funds.amount())))
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

        pub fn withdraw(&mut self,
                        caller: Proof,
                        resource: ResourceAddress,
                        amount: Decimal) -> Bucket
        {
            self.operate_on_vault(&self.proof_to_nfgid(caller),
                                  &resource,
                                  None,
                                  |mut v| Some(v.take(amount)))
                .unwrap()
        }

        pub fn withdraw_with_allowance(&mut self,
                                       allowance: Bucket,
                                       amount: Decimal)
                                       -> (Bucket, Option<Bucket>)
        {
            let allowance_resaddr = allowance.resource_address();
            let (owner_nfgid, withdraw_resource, allowance) =
                self.use_allowance(allowance, amount);

            // Note the allowance NFT may have been burned by this
            // point

            (self.operate_on_vault(
                &owner_nfgid,
                &withdraw_resource,
                Some(allowance_resaddr),
                |mut v| Some(v.take(amount)))
             .unwrap(),
             allowance)
        }

        /// NOTE we do *not* check that the allowance has the correct
        /// resource address for the pool as we have no information on
        /// the pool. This must be checked by the calling party.
        fn use_allowance(&self, allowance: Bucket, amount: Decimal)
                         -> (NonFungibleGlobalId, ResourceAddress, Option<Bucket>)
        {
            let nfdata: AllowanceNfData =
                ResourceManager::from(allowance.resource_address())
                .get_non_fungible_data(
                    &allowance.as_non_fungible().non_fungible_local_id());

            let owner_nfgid = nfdata.escrow_pool.1;
            let withdraw_resource = nfdata.for_resource;
            let mut burn = false;

            assert_eq!(Runtime::global_address(), nfdata.escrow_pool.0,
                       "allowance is not for this escrow");
            assert!(nfdata.max_amount.is_none()
                    || nfdata.max_amount.unwrap() >= amount,
                    "insufficient allowance");

            let now = unix_time_now();
            assert!(nfdata.valid_after <= now,
                    "allowance not yet valid");
            assert!(nfdata.valid_until.is_none()
                    || nfdata.valid_until.unwrap() >= now,
                    "allowance no longer valid");

            match nfdata.life_cycle {
                AllowanceLifeCycle::OneOff => {
                    burn = true;
                },
                AllowanceLifeCycle::Accumulating => {
                    let new_max = nfdata.max_amount.unwrap() - amount;
                    if new_max.is_zero() { burn = true }
                    else {
                        ResourceManager::from(allowance.resource_address())
                            .update_non_fungible_data(
                                &allowance.as_non_fungible().non_fungible_local_id(),
                                "max_amount",
                                Some(new_max));
                    }
                },
                AllowanceLifeCycle::Repeating{min_delay} => {
                    if let Some(min_delay) = min_delay {
                        ResourceManager::from(allowance.resource_address())
                            .update_non_fungible_data(
                                &allowance.as_non_fungible().non_fungible_local_id(),
                                "valid_after",
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

        pub fn withdraw_non_fungibles(&mut self,
                                      caller: Proof,
                                      resource: ResourceAddress,
                                      lids: IndexSet<NonFungibleLocalId>)
                                      -> Bucket
        {
            self.operate_on_vault(&self.proof_to_nfgid(caller),
                                  &resource,
                                  None,
                                  |v| Some(v.as_non_fungible().take_non_fungibles(&lids).into()))
                .unwrap()
        }

        pub fn withdraw_non_fungibles_with_allowance(
            &mut self,
            allowance: Bucket,
            lids: IndexSet<NonFungibleLocalId>)
            -> (Bucket, Option<Bucket>)
        {
            let allowance_resaddr = allowance.resource_address();
            let (owner_nfgid, withdraw_resource, allowance) =
                self.use_allowance(allowance, lids.len().into());

            // Note the allowance NFT may have been burned by this
            // point

            (self.operate_on_vault(
                &owner_nfgid,
                &withdraw_resource,
                Some(allowance_resaddr),
                |v| Some(v.as_non_fungible().take_non_fungibles(&lids).into()))
             .unwrap(),
             allowance)
        }

        pub fn withdraw_all_of(&mut self,
                               caller: Proof,
                               resource: ResourceAddress)
                               -> Bucket
        {
            self.operate_on_vault(&self.proof_to_nfgid(caller),
                                  &resource,
                                  None,
                                  |mut v| Some(v.take_all()))
                .unwrap()
        }

        pub fn subsidize(&mut self,
                         caller: Proof,
                         amount: Decimal)
        {
            self.operate_on_vault(&self.proof_to_nfgid(caller),
                                  &XRD,
                                  None,
                                  |v| {v.as_fungible().lock_fee(amount); None});
        }

        pub fn subsidize_with_allowance(&mut self,
                         allowance: Bucket,
                         amount: Decimal) -> Option<Bucket>
        {
            let allowance_resaddr = allowance.resource_address();
            let (owner_nfgid, withdraw_resource, allowance) =
                self.use_allowance(allowance, amount);

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

        pub fn subsidize_contingent(&mut self,
                                    caller: Proof,
                                    amount: Decimal)
        {
            self.operate_on_vault(
                &self.proof_to_nfgid(caller),
                &XRD,
                None,
                |v| {v.as_fungible().lock_contingent_fee(amount); None});
        }

        /// The owner of a pool can mint allowances to that pool, and
        /// can then distribute those allowance NFTs to whoever.
        ///
        /// The allowance will be for the pool owned by the `owner`
        /// proof. If a pool doesn't yet exist, one will be created.
        ///
        /// Otherwise, specify the parameters of the allowance (see
        /// the doc for the AllowanceNfData struct for details), and
        /// the newly created allowance will be returned out of this
        /// method.
        pub fn mint_allowance(&mut self,
                              owner: Proof,
                              valid_until: Option<i64>,
                              valid_after: i64,
                              life_cycle: AllowanceLifeCycle,
                              for_resource: ResourceAddress,
                              max_amount: Option<Decimal>) -> Bucket
        {
            assert!(max_amount.is_none()
                    || !max_amount.unwrap().is_negative(),
                    "max_amount cannot be negative");

            // Access control is effectively enforced through our pool
            // lookup further down.
            let owner = owner.skip_checking();

            let owner = NonFungibleGlobalId::new(
                owner.resource_address(),
                owner.as_non_fungible().non_fungible_local_id());

            let pool_mgr = ResourceManager::from(
                self.get_or_add_pool(&owner).allowance_badge_res);

            self.create_allowance(
                (Runtime::global_address(), owner),
                pool_mgr,
                valid_until,
                valid_after,
                life_cycle,
                for_resource,
                max_amount)
        }

        /// Anyone who holds an `allowance` NFT can voluntarily reduce
        /// its max amount by calling this method. Just provide a
        /// proof that you control the allowance and you're good.
        ///
        /// Make sure that `new_max` is lower than (or equal to) the
        /// `max_amount` currently in the allowance or this method
        /// will panic. Also don't send in a negative number.
        pub fn reduce_allowance(&self,
                                allowance: Proof,
                                new_max: Decimal)
        {
            assert!(!new_max.is_negative(),
                    "allowance can't be negative");
            
            // Access control is effectively achieved through the
            // use of the proof's resource address and nflid later.
            let allowance = allowance.skip_checking();

            let nfdata: AllowanceNfData =
                ResourceManager::from(allowance.resource_address())
                .get_non_fungible_data(&allowance.as_non_fungible().non_fungible_local_id());

            assert!(nfdata.max_amount.is_none()
                    || nfdata.max_amount.unwrap() >= new_max,
                    "allowance increase not allowed");

            ResourceManager::from(allowance.resource_address())
                .update_non_fungible_data(&allowance.as_non_fungible().non_fungible_local_id(),
                                          "max_amount",
                                          Some(new_max));
        }

        pub fn add_trusted_nfgid(&mut self,
                                 owner: Proof,
                                 add_nfgid: NonFungibleGlobalId)
        {
            let owner_nfgid = self.proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_nfgids.insert(add_nfgid, true);
        }

        pub fn remove_trusted_nfgid(&mut self,
                                    owner: Proof,
                                    remove_nfgid: NonFungibleGlobalId)
        {
            let owner_nfgid = self.proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_nfgids.insert(remove_nfgid, false);
        }

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

        pub fn add_trusted_resource(&mut self,
                                    owner: Proof,
                                    add_resource: ResourceAddress)
        {
            let owner_nfgid = self.proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_res.insert(add_resource, true);
        }

        pub fn remove_trusted_resource(&mut self,
                                       owner: Proof,
                                       remove_resource: ResourceAddress)
        {
            let owner_nfgid = self.proof_to_nfgid(owner);
            let pool = self.get_or_add_pool(&owner_nfgid);
            pool.trusted_res.insert(remove_resource, false);
        }
        
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
        // Internal helper methods follow
        //
        
        fn is_nfgid_trusted_by_pool(&self,
                                    pool: &KeyValueEntryRef<Pool>,
                                    candidate: NonFungibleGlobalId) -> bool
        {
            if let Some(trust) = pool.trusted_nfgids.get(&candidate) {
                return *trust
            }
            false
        }

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

        fn proof_to_nfgid(&self,
                          proof: Proof)
                          -> NonFungibleGlobalId
        {
            // We don't need to validate since we accept all
            // non-fungible badges.
            let owner = proof.skip_checking();
            NonFungibleGlobalId::new(
                owner.resource_address(),
                owner.as_non_fungible().non_fungible_local_id())
        }

//        fn is_trusted_nfgid(&self,
//                            pool: &Pool,
//                            nfgid: &NonFungibleGlobalId)
//                            -> bool
//        {
//            if let Some(trusted) = pool.trusted_nfgids.get(nfgid) {
//                *trusted
//            } else {
//                false
//            }
//        }
        
//        fn is_trusted_resource(&self,
//                               pool: &Pool,
//                               resource: &ResourceAddress)
//                               -> bool
//        {
//            if let Some(trusted) = pool.trusted_res.get(resource) {
//                *trusted
//            } else {
//                false
//            }
//        }

        /// NOTE when calling this method, `owner` must be a
        /// confirmed identity as it is used to grant access to
        /// vaults. It *must* have come out of a proof, or out of a
        /// bucket, or otherwise from a trusted source.
        ///
        /// If 'allowance_resaddr' is present then authority to
        /// operate on the vault came from an allowance NFT with that
        /// resource address. This method asserts that that is indeed
        /// the resource address of the pool's allowance NFTs.
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
                // they've issued.
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

        fn create_allowance(&self,
                            escrow_pool: (ComponentAddress, NonFungibleGlobalId),
                            pool_mgr: ResourceManager,
                            valid_until: Option<i64>,
                            valid_after: i64,
                            life_cycle: AllowanceLifeCycle,
                            for_resource: ResourceAddress,
                            max_amount: Option<Decimal>) -> Bucket
        {
            pool_mgr
                .mint_ruid_non_fungible(
                    AllowanceNfData {
                        escrow_pool,
                        valid_until,
                        valid_after,
                        life_cycle,
                        for_resource,
                        max_amount
                    }
                )
        }
    }
}
