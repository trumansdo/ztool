use std::collections::HashMap;

use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res =
        reqwest::get("https://randomuser.me/api/?noinfo&results=2&exc=dob,registered").await?;
    let head_map = res.headers();
    println!("headers = {head_map:?}");
    let body = res.json::<HashMap<String, Value>>().await?;
    println!("body = {body:?}");
    Ok(())
}
