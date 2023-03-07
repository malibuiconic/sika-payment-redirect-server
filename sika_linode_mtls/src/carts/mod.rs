use chrono::prelude::*;
use crate::ShopifyPaymentRequest;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ShopifyCart {
    pub id: Option<String>,  // Unique Identifier for the payment attempt from Shopify (Used as Idempotency key)
    pub timestamp: DateTime<Utc>,
    pub shopify_store: Option<String>,
    pub shopify_store_token: Option<String>,           // perm_token recv from DB (we got from app install)
    pub shopify_store_eligible: Option<Vec<String>>,   // bring in store's eligible products by comparison (this is specific to Sika - mysql db request)
    pub shopify_cart: Option<ShopifyPaymentRequest>,
}

impl ShopifyCart {
    pub fn new() -> ShopifyCart {
        ShopifyCart {
            id: None,
            timestamp: Utc::now(),
            shopify_store: None,
            shopify_store_token: None,
            shopify_store_eligible: None,
            shopify_cart: None,
        }
    }
}

/*
impl fmt::Display for ShopifyCart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "{:?}, {}", Some(self.id), self.timestamp)   
    }
}
*/
