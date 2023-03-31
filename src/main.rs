use std::{time, thread};
use reqwest::{self, StatusCode};
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

// FIXME: messy consts. Bundle paths and queries together somehow
const BUSTRACKER: (&str, &str, &str) =
    ("http://www.ctabustracker.com", "/bustime/api/v2/getvehicles", "format=json&key=<KEY>&rt=50&spid=1802");

const TRAINTRACKER: (&str, &str, &str) =
    ("http://lapi.transitchicago.com", "/api/1.0/ttarrivals.aspx", "outputType=JSON&key=<KEY>&mapid=40380");

// use reqwest::header::Authorization;
// use reqwest::header::ContentType;
#[derive(Serialize, Deserialize, Debug)]
struct CTABus {
  heading: i16,
  vehicle_id: String,
  // #[serde(with = NaiveDateTime)]
  timestamp: NaiveDateTime,
  latitude: f32,
  longitude: f32,
  route: String,
  delayed: bool,
  destination: String,
  pattern_distance: i32,
  pattern_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
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
    
    // TODO: how tf do I use argv?
    let refresh_rate = time::Duration::from_secs_f32(5.0);

    begin_tracker_loop(&client, refresh_rate).await;

    println!("Exiting... hope you caught a ride!");
}

// Main loop. Handles refreshing LED panel and fetching & parsing tracking data.
async fn begin_tracker_loop(client: &reqwest::Client, refresh_rate: time::Duration) {
    loop {
        // TODO: build structs to parse bus/train responses
        let _buses: Option<Vec<CTABus>> = cta_api_request(&client, BUSTRACKER).await;
        let _trains: Option<Vec<CTATrain>> = cta_api_request(&client, TRAINTRACKER).await;
        thread::sleep(refresh_rate);
    }
}

// Make an async request to the CTA transit tracker API of choice.
// FIXME: properly handle the generic arg.
async fn cta_api_request<T>(client: &reqwest::Client, api: (&str, &str, &str)) -> Option<Vec<T>> {
    let (endpoint, path, query) = api;
    let response =
        client.get(format!("{}{}?{}", endpoint, path, query))
        .send()
        .await
        .unwrap();

    // TODO: map buses/trains to vector of structs
    // bustime-response.vehicle = Vec<CTABus>
    // TODO: serde plus JSON? reqwest plus JSON? hmm...
    match response.status() {
        StatusCode::OK => {
            println!("{:?}", response);
            None
        },//response.json::<Vec<CTATrain>>,//Some(vec![result]),
        // Err(reqwest::StatusCode::NOT_FOUND) => println!("404 Not found"),
        // Err(reqwest::StatusCode::BAD_REQUEST) => println!("400 Bad request"),
        // Err(reqwest::StatusCode::BAD_GATEWAY) => println!("502 Bad gateway"),
        StatusCode::BAD_REQUEST => None,
        StatusCode::UNAUTHORIZED => None,
        _ => None,
    }
    // return body;
}
