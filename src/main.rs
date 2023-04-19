#![warn(missing_docs)]

//! YACTATT: Yet Another CTA Transit Tracker
//!
//! This is a simple standalone embedded Rust program designed to take
//! bus/train routes and stops/stations respectively and output them
//! in order of arrival time on an RGB LED matrix.

use std::{time::{self, SystemTime}, collections::{BTreeMap}};
use chrono::NaiveDateTime;
use clap::{arg, Command};
use reqwest::{self, StatusCode, Url, header::ACCEPT};
use rpi_led_panel::{RGBMatrix, RGBMatrixConfig, Canvas};
use serde::{Deserialize};
use embedded_graphics::{
    primitives::{Rectangle, Primitive, PrimitiveStyle, Line},
    text::Text,
    mono_font::{ascii::{FONT_9X15_BOLD, FONT_4X6}, MonoTextStyle},
    prelude::*,
    pixelcolor::Rgb888,
    Drawable
};

const BUSTRACKER_URL: &str =
    "http://www.ctabustracker.com/bustime/api/v2/getpredictions?format=json&key=<KEY>&rt=50&stpid=1802";

const TRAINTRACKER_URL: &str =
    "http://lapi.transitchicago.com/api/1.0/totarrivals.aspxoutputType=JSON&key=<KEY>&mapid=40380";

const VERTICAL_OFFSET: i32 = 6;

// use reqwest::header::Authorization;
// use reqwest::header::ContentType;

#[derive(Clone, Deserialize, Debug)]
enum Prediction {
    A,
    D
}

// TODO: use #[serde(flatten)] to remove needless nested keys

#[derive(Clone, Deserialize, Debug)]
struct CTABus {
    tmstmp: String, // "YYYYMMDD HH24:MI"
    typ: Prediction,
    stpnm: String, // Stop display name
    stpid: String, // Stop ID
    vid: String,   // Vehicle ID
    dstp: i32, // Distance to stop
    rt: String,
    rtdd: String,
    rtdir: String, // Direction (Southbound, etc.)
    des: String, // Destination
    prdtm: String, // Predicted time to arrival
    #[serde(skip)]
    tablockid: String, // TA block ID
    #[serde(skip)]
    tatripid: String, // TA trip ID
    origtatripno: String,
    dly: bool, // Delayed?
    prdctdn: String, // Time, in minutes, until bus arrives
    #[serde(skip)]
    zone: String // Empty string if vehicle has not entered a defined zone
}

#[derive(Deserialize, Debug)]
struct CTABusTimesResponse {
    #[serde(rename(deserialize = "bustime-response"))]
    bustime_response: CTABusTimesPredictions
}

#[derive(Deserialize, Debug)]
struct CTABusError {
    rt: String,
    stpid: String,
    msg: String
}

type CTABusErrors = Vec<CTABusError>;

/// Bus predictions can either have the key "prd" for predictions or "error" if something went wrong.
#[derive(Deserialize, Debug)]
struct CTABusTimesPredictions {
    #[serde(default)]
    prd: CTABusTimes,
    #[serde(default)]
    error: CTABusErrors
}

type CTABusTimes = Vec<CTABus>;

#[derive(Deserialize, Debug)]
struct CTATrain {
    delayed: bool,
    heading: i16,
    latitude: f32,
    longitude: f32,
    route: String,
    timestamp: NaiveDateTime,
    vehicle_id: String,
}

#[derive(Deserialize, Debug)]
enum CTAVehicle {
    Bus(CTABus),
    Train(CTATrain)
}

/// CTAVehicle structs can draw their own data given a matrix and a canvas.
/// This lets us loop over each struct we deserialize after calling the CTA API.
impl CTAVehicle {
    fn draw(&self, canvas: &mut Box<Canvas>, x: i32, y: i32) {
        let (route, prediction, destination) = match self {
            CTAVehicle::Bus(bus) => (String::clone(&bus.rt), String::clone(&bus.prdctdn), String::clone(&bus.des)),
            CTAVehicle::Train(train) => (String::from("red"), String::from("??"), String::from("Hell!"))
        };
        let cta_sign_grey: i32 = 0x565a5c;
        Text::new(&route, Point::new(x,y), MonoTextStyle::new(&FONT_4X6, i32_to_rgb888(0xFFA600)))
            .draw(canvas.as_mut()).unwrap();
        Text::new(&prediction, Point::new(x + 10,y), MonoTextStyle::new(&FONT_4X6, i32_to_rgb888(cta_sign_grey)))
            .draw(canvas.as_mut()).unwrap();
        Text::new(&destination, Point::new(x + 20,y), MonoTextStyle::new(&FONT_4X6, i32_to_rgb888(cta_sign_grey)))
            .draw(canvas.as_mut()).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let arguments = Command::new("YACTATT")
        .version("0.1.0") // TODO: use version in Cargo.toml
        .author("Zak Hammerman <@zatchheems>")
        .about("Yet Another CTA Transit Tracker")
        .args([
            arg!(-b --brightness <u8> "LED screen brightness").value_parser(clap::value_parser!(u8)),
            arg!(-r --rows <usize> "LED matrix rows").value_parser(clap::value_parser!(usize)),
            arg!(-c --cols <usize> "LED matrix columns").value_parser(clap::value_parser!(usize)),
            arg!(-R --refresh_rate <usize> "Refresh rate").value_parser(clap::value_parser!(usize)),
            // TODO: use existence as implicit true/false
            arg!(-s --silent <bool> "Silent (no LED display)").value_parser(clap::value_parser!(bool)),
            arg!(-t --timeout <f32> "Time between refetch").value_parser(clap::value_parser!(f32)),
        ])
        .get_matches();

    let brightness: u8 = *arguments.get_one::<u8>("brightness").unwrap_or(&7);
    let rows: usize = *arguments.get_one::<usize>("rows").unwrap_or(&16);
    let cols: usize = *arguments.get_one::<usize>("cols").unwrap_or(&64);
    let refresh: usize = *arguments.get_one::<usize>("refresh_rate").unwrap_or(&120);
    let silent: bool = *arguments.get_one::<bool>("silent").unwrap_or(&false);
    let client = reqwest::Client::new();
    let timeout = *arguments.get_one::<f32>("timeout").unwrap_or(&60.0);

    // Predeclare matrix and canvas for use later. Not initialized by default since
    // we may want to run it in "silent" mode, wherein no LED panel is used. Great for debugging.
    let (matrix, mut canvas): (RGBMatrix, Box<Canvas>);

    println!("Starting YACTATT...");

    // If not using the silent debug mode, set up LED panel framework
    if !silent {
        (matrix, canvas) = initialize_matrix(rows, cols, refresh);
        let _ = &canvas.set_brightness(brightness);
        let (matrix_, _) = initialize_matrix(rows, cols, refresh);
        splash_screen(32, 64, matrix, canvas.clone());
        begin_tracker_loop(&client, time::Duration::from_secs_f32(timeout), silent, matrix_, canvas).await;
    }

    println!("Exiting... hope you caught a ride!");
}

/// Main loop. Handles refreshing LED panel and fetching & parsing tracking data.
async fn begin_tracker_loop(
    client: &reqwest::Client,
    timeout: time::Duration,
    silent: bool,
    mut matrix: RGBMatrix,
    mut canvas: Box<Canvas>,
) -> () {
    loop {
        let now = SystemTime::now();
        match cta_api_request(&client, BUSTRACKER_URL).await {
            Some(buses) => {
                println!("{:?}", buses);
                if !silent {
                    loop {
                        let mut offset = VERTICAL_OFFSET;
                        // FIXME: crashes when prd is empty. Check .errors
                        buses.prd
                            .clone()
                            .into_iter()
                            .for_each(|bus| {
                                    CTAVehicle::draw(&CTAVehicle::Bus(bus), &mut canvas, 1, offset);
                                    offset += VERTICAL_OFFSET;
                                });
                        canvas = matrix.update_on_vsync(canvas);

                        if now.elapsed().unwrap() > timeout {
                            canvas.clear(Rgb888::new(0,0,0));
                            break;
                        }
                    }
                }
            },
            None => println!("Missing bus response..."),
        }
    }
}

/// Make an async request to the CTA transit tracker API of choice.
// FIXME: properly handle the generic arg.
async fn cta_api_request(client: &reqwest::Client, api_url: &str ) -> Option<CTABusTimesPredictions> {
    let url = Url::parse(api_url).expect("Invalid URL given.");
    let response =
        client.get(url)
        .header(ACCEPT, "application/json")
        .send()
        .await
        .unwrap();

    // TODO: map buses/trains to vector of structs
    match response.status() {
        StatusCode::OK => {
            // FIXME: no prd key exists when no service is scheduled:
            /* {
                "bustime-response": {
                    "error": [
                        {
                            "rt": "50",
                            "stpid": "1802",
                            "msg": "No service scheduled"
                        }
                    ]
                }
            }*/
            let bustimes = response.json::<CTABusTimesResponse>().await.unwrap();
            Some(bustimes.bustime_response)
        },//response.json::<Vec<CTATrain>>,//Some(vec![result]),
        // Err(reqwest::StatusCode::NOT_FOUND) => println!("404 Not found"),
        // Err(reqwest::StatusCode::BAD_GATEWAY) => println!("502 Bad gateway"),
        StatusCode::BAD_REQUEST => None,
        StatusCode::UNAUTHORIZED => None,
        _ => None,//None,
    }
    // return body;
}

/// Create a new RGBMatrix and boxed Canvas.
fn initialize_matrix(rows: usize, cols: usize, refresh_rate: usize) -> (RGBMatrix, Box<Canvas>) {
// TODO: add command-line args for all required struct fields
    let config: RGBMatrixConfig = RGBMatrixConfig{
        chain_length: 1,
        dither_bits: 0,
        hardware_mapping: rpi_led_panel::HardwareMapping::adafruit_hat_pwm(),
        interlaced: false,
        rows,
        cols,
        led_sequence: rpi_led_panel::LedSequence::Rbg,
        row_setter: rpi_led_panel::RowAddressSetterType::Direct,
        pwm_lsb_nanoseconds: 300,
        refresh_rate,
        ..RGBMatrixConfig::default()
    };
    RGBMatrix::new(config, 0).expect("Failed to initialize matrix.")
}

/// Convert 32-bit integers to Rgb888 structs for use in embedded graphics.
/// 
/// Examples:
/// 
/// ```rust
/// let coffee = i32_to_rgb888(#c0ffee);
/// assert_eq!(coffee, RGB888::new(192, 255, 254));
/// ```
fn i32_to_rgb888(color: i32) -> Rgb888 {
    // Split out color components from hex codes
    let r: u8 = ((color & 0xff0000) >> 16) as u8;
    let g: u8 = (color & 0x0000ff) as u8;
    let b: u8 = ((color & 0x00ff00) >> 8) as u8;
    Rgb888::new(r, g, b)

}

/// Draw a quick, simple test to show all CTA colors.
/// TODO:
///     - show version, device info, my handle, etc.
///     - cool animations, because why not
///     - different alternate screens:
///         + smooth gradient between colors
///         + concentric outlined boxes that build in, show text, and blink out consecutively
fn splash_screen(rows: usize, cols: usize, mut matrix: RGBMatrix, mut canvas: Box<Canvas>) {
    // TODO: static, constant CTA colors
    let cta_train_colors: BTreeMap<&str, i32> = BTreeMap::from([
        ("red", 0xc60c30),
        ("blue", 0x00a1de),
        ("brown", 0x62361b),
        ("green", 0x009b3a),
        ("orange", 0xf9461c),
        ("purple", 0x522398),
        ("pink", 0xe27ea6),
        ("yellow", 0xf9e300),
    ]);
    let cta_sign_grey: i32 = 0x565a5c;
    let mut offset: i32 = 0;
    // Record start time so we can break after showing splash screen for several seconds
    let now = SystemTime::now();
    // Draw splash screen
    loop {
        for (_name, color) in &cta_train_colors {
            let top_left: Point = Point::new(
                (offset % 4) * (cols as i32 / 4),
                if offset > 4 {rows as i32 / 2} else {0}
            );

// FIXME: you want sauce with that spaghetti?
            let rectangle =
            Rectangle::new(
                top_left,
                    Size::new(cols as u32 / 4, rows as u32 / 2)
                )
                .into_styled(PrimitiveStyle::with_fill(i32_to_rgb888(*color)));
            rectangle.draw(canvas.as_mut()).unwrap();
            { offset += 1; offset };
        }
        let start: Point = Point::new(0, rows as i32 / 2); 
        let end: Point = Point::new(cols as i32, rows as i32 / 2); 
        let text_start: Point = Point::new(1, (rows as i32 / 2) + 4); 
        Line::new(start, end)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(0,0,0), 12))
            .draw(canvas.as_mut()).unwrap();
        Text::new("YACTATT", text_start, MonoTextStyle::new(&FONT_9X15_BOLD, i32_to_rgb888(cta_sign_grey)))
            .draw(canvas.as_mut()).unwrap();
        
        // TODO: print version number and my handle
        // FIXME: HORRIBLE flicker
        canvas = matrix.update_on_vsync(canvas);
        offset = 0;
        if now.elapsed().unwrap().as_secs() > 5 {
            break;
        }
    }
}
