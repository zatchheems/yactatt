const BUSTRACKER_ENDPOINT: &str = "http://www.ctabustracker.com";
const BUSTRACKER_PATH: &str = "/bustime/api/v2/getvehicles";

const TRAINTRACKER_ENDPOINT: &str = "http://lapi.transitchicago.com";
const TRAINTRACKER_PATH: &str = "/api/1.0/ttarrivals.aspx";

// TODO: this should probably be a hash map(???)
const QUERY_PARAMETERS: &str = "outputType=JSON&key=<KEY>&mapid=40380";

use reqwest;
// use reqwest::header::Authorization;
// use reqwest::header::ContentType;

#[tokio::main]
async fn main() {
    // Set up LED panel framework
    println!("Starting YACTATT...");
    let client = reqwest::Client::new();
    fetch_train(client).await;
}

async fn fetch_train(client: reqwest::Client) {
    let response =
        client.get(format!("{}{}?{}", TRAINTRACKER_ENDPOINT, TRAINTRACKER_PATH, QUERY_PARAMETERS))
        .send()
        .await;

    match response {
        Ok(result) => println!("{:?}", result
        .text()
        .await),
        // Err(reqwest::StatusCode::NOT_FOUND) => println!("404 Not found"),
        // Err(reqwest::StatusCode::BAD_REQUEST) => println!("400 Bad request"),
        // Err(reqwest::StatusCode::BAD_GATEWAY) => println!("502 Bad gateway"),
        Err(_) => println!("oh no")
    }
    // return body;
}
