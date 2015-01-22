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
    theca [options] [-c] [-l LIMIT] [--reverse]
    theca [options] [-c] <id>
    theca [options] [-c] search [--regex, --body] <pattern>
    theca [options] transfer <id> to <name>
    theca [options] add <title> [--started|--urgent] [-b BODY|--editor|-]
    theca [options] edit <id>  [<title>|--append TEXT|--prepend TEXT] [--started|--urgent|--none] [-b BODY|--editor|-]
    theca [options] del <id>
    theca (-h | --help)
    theca --version

Options:
    -h, --help                          Show this screen.
    -v, --version                       Show the version of theca.
    --profile-folder PROFILEPATH        Path to folder containing profile.json files [default
                                        can be set with env var THECA_PROFILE_FOLDER].
    -p PROFILE, --profile PROFILE       Specify non-default profile [default can be set 
                                        with env var THECA_DEFAULT_PROFILE].
    -c, --condensed                     Use the condensed printing format.
    -e, --encrypted                     Specifies using an encrypted profile.
    -k KEY, --key KEY                   Encryption key to use for encryption/decryption,
                                        a prompt will be displayed if no key is provided.
    -y, --yes                           Silently agree to any y/n prompts.
    --regex                             Set search pattern to regex (default is plaintext).
    --body                              Search the body of notes instead of the title.
    -l LIMIT                            Limit listing to LIMIT items [default: 0].
    --datesort                          Sort items by date, can be used with --reverse.
    --none                              No status. (default)
    --started                           Started status.
    --urgent                            Urgent status.
    --append TEXT                       Append TEXT to the note title.
    --prepend TEXT                      Prepend TEXT to the note title.
    -b BODY                             Set body of the item from BODY.
    --editor                            Drop to $EDITOR to set/edit item body.
    -                                   Set body of the item from STDIN.
";

pub fn theca_main() -> Result<(), ThecaError> {
    let mut args: Args = try!(Docopt::new(USAGE)
                            .and_then(|d| d.decode()));

    try!(setup_args(&mut args));

    let (mut profile, profile_fingerprint) = try!(ThecaProfile::new(&args));

    // this could def be better
    // what root command was used
    if args.cmd_transfer {
        try!(profile.transfer_note(&args))
    } else if args.cmd_add {
        // add a item
        try!(profile.add_item(&args));
    } else if args.cmd_edit {
        // edit a item
        try!(profile.edit_item(&args));
    } else if args.cmd_del {
        // delete a item
        profile.delete_item(&args.arg_id[0]);
    } else if args.cmd_clear {
        try!(profile.clear(&args));
    } else if args.flag_v {
        // display theca version
        println!("theca v{}", VERSION);
    } else if args.cmd_search {
        // search for an item
        try!(profile.search_items(&args));
    } else if !args.arg_id.is_empty() && !args.cmd_transfer {
        // view short item
        try!(profile.view_item(&args));
    } else if args.cmd_info {
        try!(profile.stats(&args));
    } else if !args.cmd_new_profile {
        // this should be the default for nothing
        try!(profile.list_items(&args));
    }

    // save altered profile back to disk
    // this should only be triggered by commands that make alterations to the profile
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
