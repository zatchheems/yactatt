use std::{time, thread, collections::HashMap};
use chrono::NaiveDateTime;
use clap::{Arg, arg, Command, ArgMatches};
use reqwest::{self, StatusCode, Url};
use rpi_led_panel::{RGBMatrix, RGBMatrixConfig, Canvas};
use serde::{Serialize, Deserialize};

struct URL {
    protocol: String,
    domain: String,
    path: String,
    query: String
}

const BUSTRACKER_URL: &str =
    "http://www.ctabustracker.com/bustime/api/v2/getvehiclesformat=json&key=<KEY>&rt=50&spid=1802";

const TRAINTRACKER_URL: &str =
    "http://lapi.transitchicago.com/api/1.0/totarrivals.aspxoutputType=JSON&key=<KEY>&mapid=40380";

// static CTA_COLORS: HashMap<&str, i32> = HashMap::from([
//     ("red", 0xc60c30),
//     ("blue", 0x00a1de),
//     ("brown", 0x62361b),
//     ("green", 0x009b3a),
//     ("orange", 0xf9461c),
//     ("purple", 0x522398),
//     ("pink", 0xe27ea6),
//     ("yellow", 0xf9e300),
//     ("sign_grey", 0x565a5c),
// ]);

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
    let arguments = Command::new("YACTATT")
        .version("0.1.0") // TODO: use version in Cargo.toml
        .author("Zak Hammerman <@zatchheems>")
        .about("Yet Another CTA Transit Tracker")
        .args([
            arg!(-r --rows <i8> "LED matrix rows"),
            arg!(-c --cols <i8> "LED matrix columns"),
        ])
        .get_matches();


    let rows: i8 = *arguments.get_one::<i8>("rows").unwrap_or(&16);
    let cols: i8 = *arguments.get_one::<i8>("cols").unwrap_or(&64);

    // Set up LED panel framework
    println!("Starting YACTATT...");

    splash_screen(rows, cols);
    
    let client = reqwest::Client::new();
    
    // TODO: build args from env::args()
    let refresh_rate = time::Duration::from_secs_f32(5.0);

    begin_tracker_loop(&client, refresh_rate).await;

    println!("Exiting... hope you caught a ride!");
}

// Main loop. Handles refreshing LED panel and fetching & parsing tracking data.
async fn begin_tracker_loop(client: &reqwest::Client, refresh_rate: time::Duration) {
    loop {
        // TODO: build structs to parse bus/train responses
        // let _buses: Option<Vec<CTABus>> = cta_api_request(&client, BUSTRACKER).await;
        // let _trains: Option<Vec<CTATrain>> = cta_api_request(&client, TRAINTRACKER).await;
        thread::sleep(refresh_rate);
    }
}

// Make an async request to the CTA transit tracker API of choice.
// FIXME: properly handle the generic arg.
async fn cta_api_request<T>(client: &reqwest::Client, api_url: &str ) -> Option<Vec<T>> {
    let url = Url::parse(api_url).expect("Invalid URL given.");
    let response =
        client.get(url)
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

fn initalize_matrix(config: RGBMatrixConfig) -> (RGBMatrix, Box<Canvas>) {
    RGBMatrix::new(config, 0).expect("Failed to initialize matrix.")
}

fn splash_screen(rows: i8, cols: i8) {
    // let config: RGBMatrixConfig = RGBMatrixConfig::from((
    //     (hardware_mapping, ()),
    //     rows: 16,
    //     cols: 64,
    //     refresh_rate: 60,
    //     chain_length: 1,
    // ));
    // let (mut matrix, mut canvas) = initalize_matrix(config);
}
