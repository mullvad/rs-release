extern crate rs_release;

use rs_release::{get_os_release, parse_os_release, parse_os_release_str};
use std::env;

fn main() {
    let mut args = env::args();

    let os_release: Result<Vec<_>, _> = if let Some(os_release_path) = args.nth(1) {
        parse_os_release(os_release_path).and_then(|os_release| os_release.collect())
    } else {
        get_os_release().and_then(|os_release| os_release.collect())
    };

    match os_release {
        Ok(os_release) => {
            println!("Parsed os-release:");
            for (k, v) in os_release {
                println!("{}={}", k, v);
            }
        }
        Err(e) => println!("ERROR: {:?}", e),
    }

    // You could also parse data from a string
    println!("Parsed os-release from &str:");
    for (k, v) in parse_os_release_str("NAME = Fedora").filter_map(Result::ok) {
        println!("{}={}", k, v);
    }
}
