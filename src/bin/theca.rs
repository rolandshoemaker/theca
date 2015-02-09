//  _   _                    
// | |_| |__   ___  ___ __ _ 
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// licensed under the MIT license <http://opensource.org/licenses/MIT>
//
// theca.rs
//   the theca binary, we finish error unwinding in here and set
//   the exit status if there was an error.

#![feature(env)]

extern crate theca;
extern crate docopt;

use docopt::Docopt;
use theca::{Args, ThecaProfile, setup_args, parse_cmds};
use theca::errors::{ThecaError};
use std::env::{set_exit_status};

static USAGE: &'static str = "
theca - simple cli note taking tool

Usage:
    theca [options] new-profile [<name>]
    theca [options] encrypt-profile [--new-key KEY]
    theca [options] decrypt-profile
    theca [options] list-profiles
    theca [options] info
    theca [options] clear
    theca [options]
    theca [options] <id>
    theca [options] search [--regex, --search-body] <pattern>
    theca [options] transfer <id> to <name>
    theca [options] import <id> from <name>
    theca [options] add <title> [-s|-u] [-b BODY|-t|-]
    theca [options] edit <id> [<title>] [-s|-u|-n] [-b BODY|-t|-]
    theca [options] del <id>...

Profiles:
    -f PATH, --profile-folder PATH      Path to folder containing profile.json
                                        files [default can be set with env var 
                                        THECA_PROFILE_FOLDER].
    -p PROFILE, --profile PROFILE       Specify non-default profile [default
                                        can be set with env var 
                                        THECA_DEFAULT_PROFILE].

Printing format:
    -c, --condensed                     Use the condensed printing format.
    -j, --json                          Print list output as a JSON object.

Note list formatting:
    -l LIMIT, --limit LIMIT             Limit output to LIMIT notes
                                        [default: 0].
    -d, --datesort                      Sort notes by date.
    -r, --reverse                       Reverse list.

Input:
    -y, --yes                           Silently agree to any [y/n] prompts.

Statuses:
    -n, --none                          No status. (note default)
    -s, --started                       Started status.
    -u, --urgent                        Urgent status.

Body:
    -b BODY, --body BODY                Set body of the note to BODY.
    -t, --editor                        Drop to $EDITOR to set/edit note body.
    -                                   Set body of the note from STDIN.

Encryption:
    -e, --encrypted                     Specifies using an encrypted profile.
    -k KEY, --key KEY                   Encryption key to use for encryption/
                                        decryption, a prompt will be
                                        displayed if no key is provided.
    --new-key KEY                       Specifies the encryption key for a
                                        profile when using `encrypt-profile`,
                                        a prompt will be displayed if no key
                                        is provided.

Search:
    --search-body                       Search the note bodies instead of
                                        titles.

    --regex                             Set search pattern to regex (default
                                        is keyword).

Miscellaneous:
    -h, --help                          Display this help and exit.
    -v, --version                       Display the version of theca and exit.
";

fn theca_main() -> Result<(), ThecaError> {
    let mut args: Args = try!(Docopt::new(USAGE)
                            .and_then(|d| d.decode()));
    try!(setup_args(&mut args));

    let (mut profile, profile_fingerprint) = try!(ThecaProfile::new(
        &args.flag_profile,
        &args.flag_profile_folder,
        &args.flag_key,
        args.cmd_new_profile,
        args.flag_encrypted,
        args.flag_yes
    ));

    try!(parse_cmds(&mut profile, &mut args, &profile_fingerprint));

    Ok(())
}

fn main() {
    // wooo error unwinding yay
    match theca_main() {
        Err(e) => {
            println!("{}", e.desc);
            set_exit_status(1);
        },
        Ok(_) => ()
    };
}
