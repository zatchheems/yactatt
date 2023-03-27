#![forbid(unsafe_code)]

const BUSTRACKER_ENDPOINT: &str = "http://www.ctabustracker.com";
const BUSTRACKER_PATH: &str = "/bustime/api/v2/getvehicles";

const TRAINTRACKER_ENDPOINT: &str = "http://lapi.transitchicago.com";
const TRAINTRACKER_PATH: &str = "/api/1.0/ttarrivals.aspx";

fn main() {
    // Set up LED panel framework
    println!("Starting YACTATT...");
}
