// FIXME: messy consts. Bundle paths and queries together somehow
const BUSTRACKER_ENDPOINT: &str = "http://www.ctabustracker.com";
const BUSTRACKER_PATH: &str = "/bustime/api/v2/getvehicles";

const TRAINTRACKER_ENDPOINT: &str = "http://lapi.transitchicago.com";
const TRAINTRACKER_PATH: &str = "/api/1.0/ttarrivals.aspx";

// TODO: this should probably be a hash map(???)
const BUSTRACKER_QUERY: &str = "format=json&key=<KEY>&rt=50&spid=1802";
const TRAINTRACKER_QUERY: &str = "outputType=JSON&key=<KEY>&mapid=40380";

// use serde::{Serialize, Deserialize, Debug};
use reqwest;
use chrono::NaiveDate;
// use reqwest::header::Authorization;
// use reqwest::header::ContentType;
// #[derive(Serialize, Deserialize, Debug)]
struct CTABus {
  heading: i16,
  vehicle_id: String,
  timestamp: NaiveDate,
  latitude: f32,
  longitude: f32,
  route: String,
  delayed: bool,
  destination: String,
  pattern_distance: i32,
  pattern_id: i32,
}

struct CTATrain {
  heading: i16,
  vehicle_id: String,
  timestamp: String,
  latitude: f32,
  longitude: f32,
  route: String,
  delayed: bool
}

#[tokio::main]
async fn main() {
    // Set up LED panel framework
    println!("Starting YACTATT...");
    let client = reqwest::Client::new();
    // TODO: build structs to parse bus/train responses
    cta_api_request(&client, BUSTRACKER_ENDPOINT, BUSTRACKER_PATH, BUSTRACKER_QUERY).await;
    cta_api_request(&client, TRAINTRACKER_ENDPOINT, TRAINTRACKER_PATH, TRAINTRACKER_QUERY).await;
}

async fn cta_api_request(client: &reqwest::Client, endpoint: &str, path: &str, query: &str) {
    let response =
        client.get(format!("{}{}?{}", endpoint, path, query))
        .send()
        .await;

    // TODO: map buses/trains to vector of structs
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
