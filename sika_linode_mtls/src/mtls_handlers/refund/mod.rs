use warp::{ Filter, Reply, Rejection, http::StatusCode, reply::with_status,  http::HeaderMap, reply::json };
use anyhow::Result;

// refund handler (from Shopify Payments App)
pub async fn refund_handler() -> Result<impl warp::Reply, warp::Rejection>{
    //Todo!
    println!("Health Request Check!");
    Ok(StatusCode::OK)
}
