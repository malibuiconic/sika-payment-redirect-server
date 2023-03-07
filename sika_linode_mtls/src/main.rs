#![allow(unused)]
// Small Rust Warp Server (Shopify Payments App Extension Handler) - @malibu 2023

extern crate pretty_env_logger;
#[macro_use] extern crate log;
use crate::environment::EnvConfig;
use warp::{ Filter, Reply, Rejection, http::StatusCode, reply::with_status,  http::HeaderMap, reply::json };
use anyhow::Result;
use std::convert::Infallible;
use crate::carts::ShopifyCart;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use mysql::*;
use mysql::prelude::*;
use data_structs::{ ShopifyPaymentRequest, ShopifyPaymentResponse };
use mtls_handlers::payment::payment_handler;
use mtls_handlers::payment::payment_redirect;
use mtls_handlers::refund::refund_handler;

mod environment;
mod data_structs;
mod mtls_handlers;
mod carts;

type WebResult<T> = std::result::Result<T, Rejection>;
type ShopifyCarts = Arc<RwLock<HashMap<String, ShopifyCart>>>;

#[tokio::main]
async fn main() -> Result<()>{
  // Logger/Tracing
  // RUST_LOG=debug or trace cargo run
  pretty_env_logger::init();

  // Environment Variables
  let app_env = EnvConfig::env_variables()?;

  // Carts Hashmap - Since multiple stores could call this service store them
  // in a hashmap with the shopify supplied idempotency key id.
  let shopify_carts: ShopifyCarts = Arc::new(RwLock::new(HashMap::new()));

  // TODO! - Set Ports setup to allow for GCP control
  // let db_url = format!("mysql://{}:{}@localhost:{}/{}", app_env.mysql_user, app_env.mysql_password,app_env.mysql_port,app_env.mysql_database);
  // let db_pool = Pool::new(db_url.as_str())?;

  // Main MTLS Shopify Endpoints
  let payment_endpoint
  = warp::path!("payment")
  .and(warp::post())
  .and(warp::body::json())
  .and(warp::header::headers_cloned())
  .and(with_carts(shopify_carts.clone()))
  .and(with_envconfig(app_env.clone()))
  .and_then(payment_handler);

  let refund_endpoint 
    = warp::path!("refund")
    .and(warp::get())
    .and_then(refund_handler);

  let mtls_routes = payment_endpoint.or(refund_endpoint);  
  
  // SIKA STUFF
  let payment_redirect_endpoint
  = warp::path!("payment_redirect")
  .and(warp::get())
  .and(warp::query())
  .and(with_carts(shopify_carts.clone()))
  .and_then(payment_redirect);

  // Health Route - Perhaps show updates on carts active?
  let health_route = warp::path!("health").map(|| format!("{}", StatusCode::OK));
  
  let tls_routes = payment_redirect_endpoint.or(health_route);

  // Multi-Threaded Endpoints ( being either http/https/mtls )
  // Trusted CA provided by LetEncrypt (in this instance using my Linode Domain)
  // -> Every 90 days (perhaps setup a cron job systemctl on linode to rotate)
  // MTLS - This has been provided by Shopify (chained root with secondary)
  // -> https://shopify.dev/docs/apps/payments/implementation#mtls-configuration
  tokio::join!(
    // Not Encrypted
    
    // Standard TLS
    warp::serve(tls_routes)
    .tls()
    .key_path(&app_env.server_key)
    .cert_path(&app_env.server_cert)
    .run(([0,0,0,0], 7714)),
    
    // MTLS
    warp::serve(mtls_routes)
    .tls()
    .key_path(&app_env.server_key)
    .cert_path(&app_env.server_cert)
    .client_auth_required_path(&app_env.shopify_mtls)
    .run(([0,0,0,0], 443)),
  
  );
  Ok(())
}

// Helper Filter Functions
// When we need pass in ENV
fn with_envconfig(appenv: EnvConfig) -> impl Filter<Extract = (EnvConfig,), Error = Infallible> + Clone {
  warp::any().map(move || appenv.clone())
}
// When we need to pass in DB
fn with_db(db_pool: Pool) -> impl Filter<Extract = (Pool,), Error = Infallible> + Clone {
  warp::any().map(move || db_pool.clone())
}
// When we need to pass in Cache
fn with_carts(carts: ShopifyCarts) -> impl Filter<Extract = (ShopifyCarts,), Error = Infallible> + Clone {
  warp::any().map(move || carts.clone())
}





