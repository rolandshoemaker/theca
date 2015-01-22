#![allow(unstable)]
extern crate theca;

use theca::theca;

fn main() {
    // wooo error unwinding yay
    match theca() {
        Err(e) => {
            println!("{}", e.desc);
            std::os::set_exit_status(1);
        },
        Ok(_) => ()
    };
}
