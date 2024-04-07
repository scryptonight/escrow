use scrypto::prelude::*;

use crate::util::length_of_option_set;

/// This is a general way of describing how many tokens are wanted as
/// payment for some trade. Its purpose is to enable us to specify
/// *either* "n amount of fungibles" *or* "these here non-fungibles"
/// depending on which type of resource it is paired with.
#[derive(ScryptoSbor, ManifestSbor, Clone, PartialEq, Eq, Debug)]
pub enum TokenQuantity {
    /// Asks for this exact amount of a fungible token.
    Fungible(Decimal),

    /// Asks for non-fungible tokens. We can ask for a specific set of
    /// non-fungible local ids and/or we can ask for a number of
    /// arbitrarily chosen NFTs. If both parameters are in use then we
    /// are asking for the sum of those two so for example,
    /// ```NonFungible(Some(IndexSet(1,2,3)), Some(5))``` asks for
    /// nonfungible local ids 1, 2 and 3 PLUS also 5 other random NFTs
    /// of the same NFT resource. (And by random we mean arbitrary.)
    NonFungible(Option<IndexSet<NonFungibleLocalId>>, Option<u64>),
}

impl TokenQuantity {
    /// Returns zero if this quantity is for zero tokens.
    pub fn is_zero(&self) -> bool {
        match self {
            TokenQuantity::Fungible(price) => price.is_zero(),
            TokenQuantity::NonFungible(set, amount) =>
                amount.unwrap_or_default() == 0 &&
                length_of_option_set(set) == 0,
        }
    }
    
    /// Determines how many tokens in total are being asked for in
    /// an instance of the `TokenQuantity` enum.
    pub fn to_amount(&self) -> Decimal {
        match self {
            TokenQuantity::Fungible(price) => price.clone(),
            TokenQuantity::NonFungible(set, amount)
                => Decimal::from(amount.unwrap_or_default()
                                 + length_of_option_set(set) as u64),
        }
    }

    /// Determines detailed quantity specifications for this
    /// quantity. Note that this involves cloning of the data and also
    /// potentially involves a conversion from u64 to Decimal for
    /// NonFungible quantities.
    pub fn extract_max_values(&self) -> (Option<IndexSet<NonFungibleLocalId>>, Option<Decimal>)
    {
        match self {
            TokenQuantity::Fungible(price) => (None, Some(price.clone())),
            TokenQuantity::NonFungible(set, amount) => (set.clone(), amount.map(|v|Decimal::from(v))),
        }
    }

    /// Checks if the values in the provided map are consistent
    /// and make sense. Panics if this does not hold.
    pub fn check_token_quantity_sanity(map: &IndexMap<ResourceAddress, TokenQuantity>) {
        for (resaddr, ask) in map {
            let fung_res = resaddr.is_fungible();
            match ask {
                TokenQuantity::Fungible(amount) => {
                    assert!(fung_res, "fungible TokenQuantity used for non-fungible resource");
                    assert!(!amount.is_negative(), "cannot ask for negative amounts");
                },
                TokenQuantity::NonFungible(_, _) => {
                    assert!(!fung_res, "non-fungible TokenQuantity used for fungible resource");
                }
            }
        }
    }
}
