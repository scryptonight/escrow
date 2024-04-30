//! WARNING This is a mock component intended for simple
//! demonstrations with benign actors. **It has no security features
//! and should not be put into production use.**

use scrypto::prelude::*;

use radix_engine_common::ManifestSbor;
use crate::escrow::Escrow;
use crate::AllowanceNfData;
use crate::TokenQuantity;
use crate::util::proof_to_nfgid;

#[derive(ScryptoSbor, ManifestSbor, PartialEq, Eq, Debug, Hash, Clone)]
struct Actor {
    id_badge: NonFungibleGlobalId,
    escrow_payout_component: Option<ComponentAddress>,
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
enum SourceOfFunds {
    Direct{actor: Actor, price_in_xrd: Decimal, vault: Vault},
    FromEscrow{actor: Actor, price_in_xrd: Decimal, vault: Vault},
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
struct Offering
{
    source_of_funds: SourceOfFunds,
}

impl Offering {
    pub fn extract_vault(self) -> Vault {
        match self.source_of_funds {
            SourceOfFunds::Direct{vault, ..} => { vault },
            SourceOfFunds::FromEscrow{vault, ..} => { vault },
        }
    }
}

#[blueprint]
mod mock_dex {

    struct MockDex {
        meme_token: ResourceAddress,
        
        /// People who want to buy MEME are here. The Decimal is the
        /// price they want to pay, in XRD, per MEME. Their Vault
        /// contains XRD and they expect MEME back.
        ///
        /// Note that this only works because in our test scenarios
        /// there is only one offering on the book per price point. In
        /// more realistic circumstances you'd want a multimap type
        /// structure here.
        ///
        /// Also this would need to be lazy loading akin to
        /// KeyValueStore or it will lead to excessively large
        /// component state.
        buy_book: BTreeMap<Decimal, Offering>,

        /// People who want to sell MEME are here. The Decimal is the
        /// price they are willing to pay, in XRD, per MEME. Their
        /// Vault contains MEME and they expect XRD back.
        ///
        /// Note that this only works because in our test scenarios
        /// there is only one offering on the book per price point. In
        /// more realistic circumstances you'd want a multimap type
        /// structure here.
        ///
        /// Also this would need to be lazy loading akin to
        /// KeyValueStore or it will lead to excessively large
        /// component state.
        sell_book: BTreeMap<Decimal, Offering>,

        // There's no provision for payout recipients to actually
        // retrieve their funds from these vaults. Don't omit such in
        // actual production code.
        payouts_xrd: KeyValueStore<NonFungibleGlobalId, Vault>,
        payouts_meme: KeyValueStore<NonFungibleGlobalId, Vault>,
        
        /// Spent, empty, vaults go here. This is an unstable solution
        /// and should not be used in production code.
        garbage_heap: Vec<Vault>,
    }

    impl MockDex {
        pub fn instantiate_mock_dex(meme_token: ResourceAddress) -> Global<MockDex> {
            Self {
                meme_token,
                buy_book: BTreeMap::new(),
                sell_book: BTreeMap::new(),
                payouts_xrd: KeyValueStore::new(),
                payouts_meme: KeyValueStore::new(),
                garbage_heap: Vec::new(),
            }
            .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn limit_buy_direct(&mut self,
                                trader: NonFungibleProof,
                                price_in_xrd: Decimal,
                                escrow_payout_component: Option<ComponentAddress>,
                                payment: FungibleBucket) {
            assert!(payment.resource_address() == XRD, "Only XRD payment allowed for buy");
            let trader = trader.skip_checking();
            self.buy_book.insert(price_in_xrd,
                                 Offering {
                                     source_of_funds: SourceOfFunds::Direct {
                                         actor: Actor {
                                             id_badge: NonFungibleGlobalId::new(
                                                 trader.resource_address(),
                                                 trader.non_fungible_local_id()),
                                             escrow_payout_component,
                                         },
                                         price_in_xrd,
                                         vault: Vault::with_bucket(payment.into()),
                                     }});
        }

        /// Place a limit buy order. The funds you spend will be taken
        /// from the `allowance`. Funds you receive in return will
        /// either go into our payout vaults under your `trader`
        /// badge, or, if `escrow_payout_component` is set will go to
        /// that Escrow instead.
        pub fn limit_buy_with_escrow(&mut self,
                                     trader: NonFungibleProof,
                                     price_in_xrd: Decimal,
                                     escrow_payout_component: Option<ComponentAddress>,
                                     allowance: NonFungibleBucket) {
            assert_eq!(dec!(1), allowance.amount(), "Only one Allowance per trade");
            let trader = trader.skip_checking();
            self.buy_book.insert(price_in_xrd,
                                 Offering {
                                     source_of_funds: SourceOfFunds::FromEscrow {
                                         actor: Actor {
                                             id_badge: NonFungibleGlobalId::new(
                                                 trader.resource_address(),
                                                 trader.non_fungible_local_id()),
                                             escrow_payout_component,
                                         },
                                         price_in_xrd,
                                         vault: Vault::with_bucket(allowance.into()),
                                     }});
        }

        pub fn limit_sell_direct(&mut self,
                                 trader: NonFungibleProof,
                                 price_in_xrd: Decimal,
                                 escrow_payout_component: Option<ComponentAddress>,
                                 for_sale: FungibleBucket) {
            assert!(for_sale.resource_address() == self.meme_token,
                    "Only MEME can be sold");
            let trader = trader.skip_checking();
            self.sell_book.insert(price_in_xrd,
                                  Offering {
                                      source_of_funds: SourceOfFunds::Direct {
                                          actor: Actor {
                                              id_badge: NonFungibleGlobalId::new(
                                                  trader.resource_address(),
                                                  trader.non_fungible_local_id()),
                                              escrow_payout_component,
                                          },
                                          price_in_xrd,
                                          vault: Vault::with_bucket(for_sale.into()),
                                      }});
        }

        pub fn limit_sell_with_escrow(&mut self,
                                      trader: NonFungibleProof,
                                      price_in_xrd: Decimal,
                                      escrow_payout_component: Option<ComponentAddress>,
                                      allowance: NonFungibleBucket) {
            assert_eq!(dec!(1), allowance.amount(), "Only one Allowance per trade");
            let trader = trader.skip_checking();
            self.sell_book.insert(price_in_xrd,
                                  Offering {
                                      source_of_funds: SourceOfFunds::FromEscrow {
                                          actor: Actor {
                                              id_badge: NonFungibleGlobalId::new(
                                                  trader.resource_address(),
                                                  trader.non_fungible_local_id()),
                                              escrow_payout_component,
                                          },
                                          price_in_xrd,
                                          vault: Vault::with_bucket(allowance.into()),
                                      }});
        }

        /// Pay a bunch of XRD to obtain MEME.
        pub fn market_buy_direct(&mut self,
                                 trader: Option<NonFungibleProof>,
                                 escrow_payout_component: Option<ComponentAddress>,
                                 mut payment: FungibleBucket)
                                 -> (Option<FungibleBucket>, FungibleBucket)
        {
            assert_eq!(XRD, payment.resource_address(),
                       "Can only pay with XRD");
            let mut purchased = FungibleBucket::new(self.meme_token);

            let mut maker_payouts = Vec::new();
            let mut offers_to_remove = Vec::new();

            for offering in self.sell_book.iter_mut() {
                match &mut offering.1.source_of_funds {
                    SourceOfFunds::Direct { actor, price_in_xrd, vault } => {

                        let max_avail = vault.amount();
                        
                        let funds_wanted = payment.amount() / *price_in_xrd;
                        if funds_wanted.is_zero() { break; }
                        let funds_to_take = std::cmp::min(funds_wanted, max_avail);

                        // pull funds from limit vault
                        let meme_bucket = vault.take(funds_to_take);
                        
                        // xfer payment to maker
                        maker_payouts.push((actor.clone(),
                                            payment.take(meme_bucket.amount() * *price_in_xrd)));
                        purchased.put(meme_bucket.as_fungible());
                        if vault.is_empty() {
                            offers_to_remove.push(*offering.0);
                        }
                    },
                    SourceOfFunds::FromEscrow { actor, price_in_xrd, ref mut vault } => {
                        let allowance_nfgid = NonFungibleGlobalId::new(
                            vault.resource_address(),
                            vault.as_non_fungible().non_fungible_local_id());
                        let nfdata: AllowanceNfData =
                            ResourceManager::from(allowance_nfgid.resource_address())
                            .get_non_fungible_data(
                                &allowance_nfgid.local_id());

                        let max_avail = Self::find_smallest(
                            Self::find_allowance_limit(&allowance_nfgid),
                            Self::call_escrow_read_funds(nfdata.escrow_pool.clone(),
                                                         self.meme_token));

                        let funds_wanted = payment.amount() / *price_in_xrd;
                        if funds_wanted.is_zero() { break; }
                        let funds_to_take = std::cmp::min(funds_wanted, max_avail);
                        let allowance_bucket = vault.as_non_fungible().take_all();

                        // pull funds from escrow
                        let (meme_bucket, allowance_bucket) =
                            Self::call_escrow_withdraw_with_allowance(
                                nfdata.escrow_pool,
                                allowance_bucket,
                                funds_to_take);
                        
                        // xfer payment to maker
                        maker_payouts.push((actor.clone(),
                                            payment.take(meme_bucket.amount() * *price_in_xrd)));
                        purchased.put(meme_bucket.as_fungible());

                        if let Some(allowance_bucket) = allowance_bucket {
                            vault.put(allowance_bucket);
                        } else {
                            // Allowance no longer exists, remove this offering
                            offers_to_remove.push(*offering.0);
                        }
                    },
                }
            }

            for price_point in offers_to_remove {
                let offering = self.sell_book.remove(&price_point).unwrap();
                self.garbage_heap.push(offering.extract_vault());
            }

            for (actor, bucket) in maker_payouts {
                self.pay_maker(&actor, bucket);
            }

            let return_bucket;
            if let Some(escrow_payout_component) = escrow_payout_component {
                let trader = proof_to_nfgid(&trader.unwrap().skip_checking());
                Self::call_escrow_deposit_funds((escrow_payout_component, trader),
                                                purchased.into());
                return_bucket = None;
            } else {
                return_bucket = Some(purchased);
            }
            
            (return_bucket, payment)
        }
        
        /// Sell a bunch of MEME to obtain XRD.
        pub fn market_sell_direct(&mut self,
                                  trader: Option<NonFungibleProof>,
                                  escrow_payout_component: Option<ComponentAddress>,
                                  mut selling: FungibleBucket)
                                  -> (Option<FungibleBucket>, FungibleBucket)
        {
            assert_eq!(self.meme_token, selling.resource_address(),
                       "Can only sell MEME");
            let mut purchased = FungibleBucket::new(XRD);

            let mut maker_payouts = Vec::new();
            let mut offers_to_remove = Vec::new();

            for offering in self.buy_book.iter_mut().rev() {
                match &mut offering.1.source_of_funds {
                    SourceOfFunds::Direct { actor, price_in_xrd, vault } => {

                        let max_avail = vault.amount();
                        
                        let funds_wanted = selling.amount() * *price_in_xrd;
                        if funds_wanted.is_zero() { break; }
                        let funds_to_take = std::cmp::min(funds_wanted, max_avail);

                        // pull funds from limit vault
                        let xrd_bucket = vault.take(funds_to_take);
                        
                        // xfer payment to maker
                        maker_payouts.push((actor.clone(),
                                            selling.take(xrd_bucket.amount() / *price_in_xrd)));
                        purchased.put(xrd_bucket.as_fungible());
                        if vault.is_empty() {
                            offers_to_remove.push(*offering.0);
                        }
                    },
                    SourceOfFunds::FromEscrow { actor, price_in_xrd, ref mut vault } => {
                        let allowance_nfgid = NonFungibleGlobalId::new(
                            vault.resource_address(),
                            vault.as_non_fungible().non_fungible_local_id());
                        let nfdata: AllowanceNfData =
                            ResourceManager::from(allowance_nfgid.resource_address())
                            .get_non_fungible_data(
                                &allowance_nfgid.local_id());

                        let max_avail = Self::find_smallest(
                            Self::find_allowance_limit(&allowance_nfgid),
                            Self::call_escrow_read_funds(nfdata.escrow_pool.clone(),
                                                         XRD));

                        let funds_wanted = selling.amount() * *price_in_xrd;
                        if funds_wanted.is_zero() { break; }
                        let funds_to_take = std::cmp::min(funds_wanted, max_avail);
                        let allowance_bucket = vault.as_non_fungible().take_all();

                        // pull funds from escrow
                        let (xrd_bucket, allowance_bucket) =
                            Self::call_escrow_withdraw_with_allowance(
                                nfdata.escrow_pool,
                                allowance_bucket,
                                funds_to_take);
                        
                        // xfer payment to maker
                        maker_payouts.push((actor.clone(),
                                            selling.take(xrd_bucket.amount() / *price_in_xrd)));
                        purchased.put(xrd_bucket.as_fungible());

                        if let Some(allowance_bucket) = allowance_bucket {
                            vault.put(allowance_bucket);
                        } else {
                            // Allowance no longer exists, remove this offering
                            offers_to_remove.push(*offering.0);
                        }
                    },
                }
            }

            for price_point in offers_to_remove {
                let offering = self.buy_book.remove(&price_point).unwrap();
                self.garbage_heap.push(offering.extract_vault());
            }

            for (actor, bucket) in maker_payouts {
                self.pay_maker(&actor, bucket);
            }

            let return_bucket;
            if let Some(escrow_payout_component) = escrow_payout_component {
                let trader = proof_to_nfgid(&trader.unwrap().skip_checking());
                Self::call_escrow_deposit_funds((escrow_payout_component, trader),
                                                purchased.into());
                return_bucket = None;
            } else {
                return_bucket = Some(purchased);
            }
            
            (return_bucket, selling)
        }

        fn put_in_payout_vault(vaults: &mut KeyValueStore<NonFungibleGlobalId, Vault>,
                               recipient: &NonFungibleGlobalId,
                               funds: FungibleBucket) {
            if vaults.get(recipient).is_none() {
                vaults.insert(recipient.clone(), Vault::new(funds.resource_address()));
            }
            vaults.get_mut(recipient).unwrap().put(funds.into());
        }
        
        fn pay_maker(&mut self, maker: &Actor, payment: FungibleBucket) {
            if let Some(escrow_component) = maker.escrow_payout_component {
                Self::call_escrow_deposit_funds((escrow_component, maker.id_badge.clone()),
                                                payment.into());
            } else {
                if payment.resource_address() == XRD {
                    Self::put_in_payout_vault(&mut self.payouts_xrd, &maker.id_badge, payment);
                } else {
                    Self::put_in_payout_vault(&mut self.payouts_meme, &maker.id_badge, payment);
                }
            }
        }

        
        
        /// Finds how much can be taken when an allowance is `limit`
        /// restricted, and there is `available` funds actually
        /// available to take.
        ///
        /// `limit` is `None` for no limit (i.e. infinite limit).
        fn find_smallest(limit: Option<Decimal>, available: Decimal) -> Decimal
        {
            if let Some(limit) = limit {
                std::cmp::min(limit, available)
            } else {
                available
            }
        }
        
        fn find_allowance_limit(allowance: &NonFungibleGlobalId) -> Option<Decimal>
        {
            let nfdata: AllowanceNfData =
                ResourceManager::from(allowance.resource_address())
                .get_non_fungible_data(
                    &allowance.local_id());
            if nfdata.is_valid() {
                nfdata.max_amount.map(|v| v.to_amount())
            } else { Some(dec!(0)) }
        }
        
        fn call_escrow_read_funds(escrow: (ComponentAddress, NonFungibleGlobalId),
                                  resource: ResourceAddress) -> Decimal {
            let escrow_component: Global<Escrow> = Global::from(escrow.0);
            escrow_component.read_funds(escrow.1, resource)
        }

        fn call_escrow_withdraw_with_allowance(escrow: (ComponentAddress, NonFungibleGlobalId),
                                               allowance: NonFungibleBucket,
                                               amount: Decimal)
                                               -> (Bucket, Option<Bucket>)
        {
            let escrow_component: Global<Escrow> = Global::from(escrow.0);

            escrow_component.withdraw_with_allowance(allowance.into(),
                                                     TokenQuantity::Fungible(amount))
        }

        fn call_escrow_deposit_funds(escrow: (ComponentAddress, NonFungibleGlobalId),
                                     funds: Bucket)
        {
            let escrow_component: Global<Escrow> = Global::from(escrow.0);

            escrow_component.deposit_funds(escrow.1, funds, None);
        }
    }

}
