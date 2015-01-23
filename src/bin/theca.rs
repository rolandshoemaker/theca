#![allow(unstable)]
extern crate theca;
extern crate docopt;

use docopt::Docopt;
use theca::{Args, ThecaProfile, setup_args};
use theca::errors::{ThecaError};

pub static VERSION:  &'static str = "0.6.0-dev";

static USAGE: &'static str = "
theca - cli note taking tool

Usage:
    theca [options] new-profile <name>
    theca [options] info
    theca [options] clear
    theca [options] [-c] [-l LIMIT] [-r]
    theca [options] [-c] <id>
    theca [options] [-c] search [--regex, --search-body] <pattern>
    theca [options] transfer <id> to <name>
    theca [options] add <title> [-s|-u] [-b BODY|--editor|-]
    theca [options] edit <id>  [<title>|-a TEXT|-p TEXT] [-s|-u|-n] [-b BODY|--editor|-]
    theca [options] del <id>

Profiles:
    -pf PROFILEPATH                     Path to folder containing profile.json
                                        files [default can be set with env var 
                                        THECA_PROFILE_FOLDER].
    --profile-folder PROFILEPATH
    -p PROFILE, --profile PROFILE       Specify non-default profile [default
                                        can be set with env var 
                                        THECA_DEFAULT_PROFILE].

Printing format:
    -c, --condensed                     Use the condensed printing format.

Note list formatting:
    -l LIMIT                            Limit listing to LIMIT items
                                        [default: 0].
    -r, --reverse                       Reverse list.
    -d, --datesort                      Sort items by date, can be used with
                                        --reverse.

Input:
    -y, --yes                           Silently agree to any y/n prompts.
    -m, --merge                         Silently agree to any merge profile
                                        changes prompt.

Title:
    -a TEXT, --append TEXT              Append TEXT to the note title.
    -p TEXT, --prepend TEXT             Prepend TEXT to the note title.

Statuses:
    -n, --none                          No status. (default)
    -s, --started                       Started status.
    -u, --urgent                        Urgent status.

Body:
    -b BODY                             Set body of the item from BODY.
    --editor                            Drop to $EDITOR to set/edit item body.
    -                                   Set body of the item from STDIN.

Encryption:
    -e, --encrypted                     Specifies using an encrypted profile.
    -k KEY, --key KEY                   Encryption key to use for
                                        encryption/decryption, a prompt will be
                                        displayed if no key is provided.

Search:
    --search-body                       Search the body of notes instead of
                                        the title.
    --regex                             Set search pattern to regex (default
                                        is plaintext).

Miscellaneous:
    -h, --help                          Display this help and exit.
    -v, --version                       Display the version of theca and exit.
";

fn parse_cmds(profile: &mut ThecaProfile, args: &Args) -> Result<(), ThecaError> {
    // misc
    if args.flag_v { println!("theca v{}", VERSION); return Ok(()) }

    // add/edit/del
    if args.cmd_add || args.cmd_edit || args.cmd_del {
        if args.cmd_add { try!(profile.add_item(args)); return Ok(()) }
        if args.cmd_edit { try!(profile.edit_item(args)); return Ok(()) }
        if args.cmd_del { profile.delete_item(&args.arg_id[0]); return Ok(()) }
    }

    // transfer
    if args.cmd_transfer { try!(profile.transfer_note(args)); return Ok(()) }

    // clear
    if args.cmd_clear { try!(profile.clear(args)); return Ok(()) }

    // search
    if args.cmd_search { try!(profile.search_items(args)); return Ok(()) }

    // view
    if !args.arg_id.is_empty() { try!(profile.view_item(args)); return Ok(()) }

    // stats
    if args.cmd_info { try!(profile.stats(args)); return Ok(()) }

    // list
    if !args.cmd_new_profile { try!(profile.list_items(args)); return Ok(()) }

    Ok(())
}

fn theca_main() -> Result<(), ThecaError> {
    let mut args: Args = try!(Docopt::new(USAGE)
                            .and_then(|d| d.decode()));

    try!(setup_args(&mut args));

    let (mut profile, profile_fingerprint) = try!(ThecaProfile::new(&args));

    try!(parse_cmds(&mut profile, &args));

    // save altered profile back to disk
    // this should only be triggered by commands that make
    // alterations to the profile
    if args.cmd_add || args.cmd_edit || args.cmd_del || args.cmd_new_profile ||
       args.cmd_clear || args.cmd_transfer {
        try!(profile.save_to_file(&args, &profile_fingerprint));
    }
    Ok(())
}

fn main() {
    // wooo error unwinding yay
    match theca_main() {
        Err(e) => {
            println!("{}", e.desc);
            std::os::set_exit_status(1);
        },
        Ok(_) => ()
    };
}
