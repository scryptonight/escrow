//! A few utility functions.
//!
//! Many of these only exist because I got tired of writing out the
//! details in full every time.

use scrypto::prelude::*;

/// Reduces a PreciseDecimal down to Decimal precision by rounding to
/// the nearest 18 digits and truncating. This is useful when you need
/// to use a PreciseDecimal to move funds around.
pub fn pdec_to_dec(pdec: PreciseDecimal) -> Decimal {
    pdec.checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero).unwrap()
        .try_into().unwrap()
}

/// Converts a `Decimal` to a `u64`. Make sure the input `d`
/// is already a whole number before calling this function.
pub fn dec_to_u64(d: Decimal) -> u64 {
    d.try_into().unwrap()
}

/// Helper function to determine the current UNIX-era time.
pub fn unix_time_now() -> i64 {
    Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch
}

/// Extracts token details from a singleton proof and returns its
/// non-fungible global id.
pub fn proof_to_nfgid(proof: &CheckedNonFungibleProof) -> NonFungibleGlobalId {
    NonFungibleGlobalId::new(proof.resource_address(), proof.non_fungible_local_id())
}

/// Skips checking of an unchecked singleton proof and returns its
/// non-fungible global id.
pub fn unchecked_proof_to_nfgid(proof: Proof)
                                -> NonFungibleGlobalId
{
    // We don't need to validate since we accept all
    // non-fungible badges.
    let owner = proof.skip_checking();
    proof_to_nfgid(&owner.as_non_fungible())
}

/// Converts a u64 into an integer non-fungible local id.
pub fn u64_to_nflid(id: u64) -> NonFungibleLocalId {
    NonFungibleLocalId::Integer(id.into())
}

/// Converts a vector of buckets to a map of vaults containing
/// those buckets, with their resource address as key. The
/// input vector is cannibalized in order to create the map.
///
/// Note that if the input contains multiple buckets of the
/// same resource then that resource's map entry will contain
/// the total from all those buckets.
pub fn bucket_vec_to_vault_map(buckets: Vec<Bucket>) -> IndexMap<ResourceAddress, Vault> {
    let mut vaults = IndexMap::new();
    for b in buckets {
        if !vaults.contains_key(&b.resource_address()) {
            vaults.insert(b.resource_address().clone(),
                          Vault::with_bucket(b));
        } else {
            vaults.get_mut(&b.resource_address()).unwrap().put(b);
        }
    }
    vaults
}

/// Puts the contents of `bucket` into the correct entry of
/// `vault_map`, creating that entry if necessary.
pub fn put_bucket_in_vault_map(vault_map: &mut IndexMap<ResourceAddress, Vault>,
                           bucket: Bucket) {
    let resaddr = &bucket.resource_address();
    if vault_map.contains_key(resaddr) {
        vault_map.get_mut(resaddr).unwrap().put(bucket);
    } else {
        vault_map.insert(resaddr.clone(), Vault::with_bucket(bucket));
    }
}

/// Puts the contents of `bucket` into the correct entry of
/// `bucket_map`, creating that entry if necessary.
pub fn _put_bucket_in_bucket_map(bucket_map: &mut IndexMap<ResourceAddress, Bucket>,
                             bucket: Bucket) {
    let resaddr = &bucket.resource_address();
    if bucket_map.contains_key(resaddr) {
        bucket_map.get_mut(resaddr).unwrap().put(bucket);
    } else {
        bucket_map.insert(resaddr.clone(), bucket);
    }
}

/// Converts a vector of buckets to a map of buckets indexed
/// by the resource address of each bucket.
pub fn bucket_vec_to_bucket_map(vector: Vec<Bucket>) -> IndexMap<ResourceAddress, Bucket> {
    let mut map: IndexMap<ResourceAddress, Bucket> = IndexMap::new();
    for bucket in vector {
        let entry = map.get_mut(&bucket.resource_address());
        if entry.is_none() {
            map.insert(bucket.resource_address(), bucket);
        } else {
            entry.unwrap().put(bucket);
        }
    }
    map
}

/// Unwraps an option'd set and returns its length. Returns 0
/// when the option is None.
pub fn length_of_option_set<T>(set: &Option<IndexSet<T>>) -> usize {
    match set {
        Some(s) => s.len(),
        None => 0,
    }
}

