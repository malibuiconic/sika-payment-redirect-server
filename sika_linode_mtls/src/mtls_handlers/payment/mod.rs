use crate::WebResult;
use warp::{ Filter, Reply, Rejection, http::StatusCode, reply::with_status,  http::HeaderMap, reply::json };
use anyhow::Result;
use serde::Deserialize;
use chrono::prelude::*;
use crate::environment::EnvConfig;
use crate::carts::ShopifyCart;
use crate::ShopifyCarts;
use crate::data_structs::{LineItem, PaymentMethod};
use crate::{ ShopifyPaymentRequest, ShopifyPaymentResponse };

#[derive(Deserialize, Debug, Clone)]
pub struct PaymentID {
    id: String, 
}


// This Function Takes in the Request From Shopify Initally - this is set in the Partners Dash (app ext version area - needs to be approved!)
// - checks the active carts if id exists? (to maintain idempontency)
// - adds to active carts if id doesn't exist
// - returns url with status OK back to Shopify Checkout
pub async fn payment_handler(payment_req: ShopifyPaymentRequest, headers: HeaderMap, carts: ShopifyCarts, app_env: EnvConfig) -> Result<impl warp::Reply, warp::Rejection>{
    info!("Payment Request From Shopify Store: {:?}", headers.get("shopify-shop-domain").unwrap());
    // println!("{:?}", headers);
    println!("Payment Req ID: {}", payment_req.id);
    let check_id = carts.read().await.get(&payment_req.id).cloned();
    match check_id {
        Some(cart) => {
            // If ID is found, means its an active cart, so for idempotency just return
            // this ID as it is already active ;) -- Again Shopify will be supplying these in the payload for session mgmt
            let redirect_url = ShopifyPaymentResponse {
                redirect_url: format!("https://{}:7714/payment_redirect?id={}", app_env.server_domain, payment_req.id.clone())
            };
            Ok(with_status(json(&redirect_url), StatusCode::OK))
        },
        None => {
            // If ID NOT Found add to Carts Info From supplied payload, etc stuff
            // call database with shop name to get perm_token - TODO!
            // with db also get product list? Talking point.. if db gives error lets say?
            let perm_token = "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";
            // let shop_name = headers.get("shopify-shop-domain").unwrap(); write parser
            
            carts.write().await.insert(
                payment_req.id.clone(),
                ShopifyCart {
                    id: Some(payment_req.id.clone()),
                    timestamp: Utc::now(),
                    shopify_store: Some(String::from("ashleyfulks-devstore.myshopify.com")),
                    shopify_store_token: Some(perm_token.to_string()),
                    shopify_store_eligible: None,
                    shopify_cart: Some(payment_req.clone()),
                }
             );

            let redirect_url = ShopifyPaymentResponse {
                redirect_url: format!("https://{}:7714/payment_redirect?id={}", app_env.server_domain, payment_req.id.clone())
            };
            Ok(with_status(json(&redirect_url), StatusCode::OK))
        }
    }
}


// This is our Visual Component and Main Access Point to the GO APIS for SIKA and will be displayed in the Shopify Checkout
// - Sika Elements in here (or essentially a customer input form)
// - Will make calls to the GO API endpoint(s) where applicable that will return a status of
//   good to go? Resolve/Reject
// - Once Recived the response from the GO APIS we constuct a GraphQL answer back to Shopify, here
//   we should have the resolver to check status and provide expoential backoff. If not
//   a first response, then spin off threads to handle this backoff. It should also remove
//   from carts cache upon completion to not continue with that cart. (Note: Shopify had some custom responses for delays)
// - Once Shopify has provided an Success Response return customer with redirect back 
//   to shopify checkout to finish out the transaction. 
pub async fn payment_redirect(payment_id: PaymentID, carts: ShopifyCarts) -> WebResult<impl Reply>{
    let active_cart = carts.read().await.get(&payment_id.id).cloned();
    let mut idempontency_id = String::new();
    let mut gid = String::new();
    let mut test = false;
    let mut amount = String::new();
    let mut line_items = vec![];
    let mut cancel_url = String::new();
    
    match active_cart {
        Some(cart) => {
            let cart_struct: ShopifyCart = cart; // look at cart structs -> payload from shopify if wanting info
            // println!("{:?}", cart_struct.shopify_cart);
            // println!("{:?}", cart_struct.id);
            match cart_struct.id {
                Some(id) => {idempontency_id = id},
                None => (),
            }
            match cart_struct.shopify_cart {
                Some(cart) => {
                    println!("{:?}", cart.gid);
                    gid = cart.gid;
                    test = cart.test;
                    amount = cart.amount;
                    let payment_method: PaymentMethod = cart.payment_method;
                    cancel_url = payment_method.data.cancel_url;

                    match cart.line_items {
                        Some(lineitems) => {
                           for item in lineitems {
                               line_items.push(item);
                           }
                        },
                        None => (),
                    }
                },
                None => (),
            }
            println!("{:?}", line_items);
            


            let payment_html = format!(
                "<!DOCTYPE html>
                <html lang=\"en\">
                <head>
                    <title>Payments Modal/Page</title>
                </head>
                <style>
                    button {{
                      width: 150px;
                      height: 50px;
                      text-align: center;
                      font-size: 20px;
                      margin: 10px;
                      cursor: pointer;
                    }}
                    button:hover{{
                      opacity: 0.5;
                    }}
                </style>
                <body>
                    <h1>Payments Modal/Page</h1>
                    <div id=\"payload\">{:?} gid: {}</div>
                    <div id=\"action\"></div>
                    <p>
                        Simulations page - here we speak to GO API depending
                        on response we would trigger one of these buttons
                        which functions make the GraphQL call back to Shopify.
            
                        Note: Todo!
                        - retry policy function for when we are waiting on Shopify
                        - handle idempotency
                        - address the cancel_url if the customer chooses to cancel in your flow
                    </p>
                          
                    <button type=\"button\" style=\"background-color:lightgreen;\" id=\"resolve\">Resolve</button><br/>
                    <button type=\"button\" style=\"background-color:#FF6863;\" id=\"reject\">Reject</button><br/>
                    <button type=\"button\" style=\"background-color:lightgray;\" id=\"cancel\">Cancel</button><br/>
                </body>
                <script>
                const action = document.getElementById('action');
                const uri = '';
         
                resolve.onclick = function(){{
                  action.innerHTML = '<p><em>resolve action clicked!</em></p>';
                }}
         
                reject.onclick = function(){{
                  action.innerHTML = '<p><em>reject action clicked!</em></p>';
                }}
         
                cancel.onclick = function(){{
                  action.innerHTML = '<p><em>cancel action clicked!</em></p>';
                  window.location.href = '{}';
                }}
                </script>
            </html>", line_items, gid, cancel_url  
            );
            
            Ok(warp::reply::html(payment_html))
            
        }
        None => {
            Ok(warp::reply::html(String::from("<div>Checkout Redirect Failed! - Contact Shop/Dev</div>")))
        }
    }
    
}