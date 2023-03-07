use serde::{Deserialize, Serialize};
use chrono::prelude::*;

// MTLS Payment Request/Response From Shopify
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShopifyPaymentRequest {
    pub id: String,                   // This is the payment ID we could use to search orders later
    pub gid: String,
    pub group: String,
    pub amount: String,
    pub currency: String,
    pub test: bool,
    pub merchant_locale: String,
    pub payment_method: PaymentMethod,
    pub proposed_at: DateTime<Utc>,   // ISO8601
    pub customer: ShopifyCustomer,
    pub kind: String,                 // will default to sale 
                                      // (if authorization we need - open other endpoints of capture and void ) 
    pub line_items: Option<Vec<LineItem>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LineItem {
   pub quantity: u32,
   pub name: String,
   pub price: String,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PaymentMethod {
   #[serde(rename="type")]
   pub payment_type: String,             //["offsite"]
   pub data: PaymentMethodData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PaymentMethodData {
   pub cancel_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShopifyCustomer {
   pub email: Option<String>,                          // Required if phone number not present
   pub phone_number: Option<String>,                   // Required if email not present
   pub locale: String,                         // Required (ISO 639-1 > language) / (ISO 3166-1 Alpha-2 > country)
   pub billing_address: ShopifyCustomerAddress,
   pub shipping_address: ShopifyCustomerAddress,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShopifyCustomerAddress {
   pub given_name: Option<String>,
   pub family_name: String,                      //required
   pub line1: String, 
   pub line2: Option<String>,
   pub city: String,
   pub postal_code: Option<String>,
   pub province: Option<String>,
   pub country_code: String,
   pub company: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ShopifyPaymentResponse{
   pub redirect_url: String,
}

// Other Structs