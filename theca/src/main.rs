extern crate serialize;
extern crate time;
extern crate docopt;
use serialize::{Encodable, Decodable, Encoder, json};
use time::{now_utc, strftime};
use docopt::Docopt;
use std::os;
use std::io::fs::PathExtensions;
use std::io::{File, Truncate, Write};

static USAGE: &'static str = "
theca - cli note taking tool

Usage:
    theca new_profile
    theca new_profile <name> [--encrypted --dropbox]
    theca [options] [-c | -e] [-l LIMIT]
    theca [options] [-c | -e] <id>
    theca [options] [-c | -e] view <id>
    theca [options] add <title> [-sn | -ss | -su] [-b BODY | --editor | -]
    theca [options] edit <id> [-sn | -ss | -su] [-b BODY | --editor | -]
    theca [options] del <id>
    theca (-h | --help)
    theca --version

Options:
    -h, --help                          Show this screen.
    -v, --version                       Show the version of theca.
    --config CONFIGPATH                 Path to .thecarc configuration file.
    --profiles-folder PROFILEPATH       Path to folder container profile.json files.
    -p PROFILE, --profile PROFILE       Specify non-default profile.
    -c, --condensed                     Use the condensed print format.
    -e, --expanded                      Use the expanded print format.
    --encrypted                         Encrypt new profile, theca will prompt you for a key.
    -l LIMIT                            Limit listing to LIMIT items.
    -sn                                 No status.
    -ss                                 Started status.
    -su                                 Urgent status.
    -b BODY                             Set body of the item to BODY.
    -                                   Set body of the item to STDIN.
    --editor                            Drop to $EDITOR to set/edit item body.
";

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

#[deriving(Decodable)]
pub struct ThecaItem {
    id: int,
    title: String,
    status: String,
    body: String,
    last_touched: String
}

impl <S: Encoder<E>, E> Encodable<S, E> for ThecaItem {
        fn encode(&self, encoder: &mut S) -> Result<(), E> {
                match *self {
                        ThecaItem{id: ref p_id, title: ref p_title, status: ref p_status, body: ref p_body, last_touched: ref p_last_touched} => {
                                encoder.emit_struct("ThecaItem", 1, |encoder| {
                                    try!(encoder.emit_struct_field("id", 0u, |encoder| p_id.encode(encoder)));
                                    try!(encoder.emit_struct_field("title", 1u, |encoder| p_title.encode(encoder)));
                                    try!(encoder.emit_struct_field("status", 2u, |encoder| p_status.encode(encoder)));
                                    try!(encoder.emit_struct_field("body", 3u, |encoder| p_body.encode(encoder)));
                                    try!(encoder.emit_struct_field("last_touched", 4u, |encoder| p_last_touched.encode(encoder)));
                                    Ok(())
                            })
                    }
            }
    }
}

#[deriving(Decodable)]
pub struct ThecaProfile {
    current_id: int,
    encrypted: bool,
    notes: Vec<ThecaItem>
}

// impl ThecaItem {
//     fn decode_item(&mut self, id: int, key: String) {
//     }
//     fn format_item(&mut self, id: int) {
//     }
// }

impl <S: Encoder<E>, E> Encodable<S, E> for ThecaProfile {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        match *self {
            ThecaProfile{current_id: ref p_current_id, encrypted: ref p_encrypted, notes: ref p_notes} => {
                encoder.emit_struct("ThecaProfile", 1, |encoder| {
                    try!(encoder.emit_struct_field("current_id", 0u, |encoder| p_current_id.encode(encoder)));
                    try!(encoder.emit_struct_field("encrypted", 1u, |encoder| p_encrypted.encode(encoder)));
                    try!(encoder.emit_struct_field("notes", 2u, |encoder| p_notes.encode(encoder)));
                    Ok(())
                })
            }
        }
    }
}

impl ThecaProfile {
    fn save_to_file(&mut self, args: &docopt::ArgvMap) {
        // set profile folder
        let mut profile_path = if args.get_bool("--profiles-folder") {
            Path::new(args.get_str("PROFILEPATH"))
        } else {
            match os::homedir() {
                Some(ref p) => p.join(".theca"),
                None => Path::new(".").join(".theca")
            }
        };

        // set file name
        if args.get_bool("-p") {
            profile_path.push(args.get_str("-p").to_string() + ".json");
        } else if args.get_bool("new_profile") {
            profile_path.push(args.get_str("<name>").to_string() + ".json");
        } else {
            profile_path.push("default".to_string() + ".json");
        }

        // save to file
        let mut file = match File::open_mode(&profile_path, Truncate, Write) {
            Ok(f) => f,
            Err(e) => panic!("File error: {}", e)
        };

        let mut encoder = json::PrettyEncoder::new(&mut file);
        // let mut encoder = json::Encoder::new(&mut file);
        self.encode(&mut encoder).unwrap();
    }

    fn add_item(&mut self, a_title: String, a_status: String, a_body: String) {
        match self.encrypted {
            true => {
                // uh not this, but placeholder for now!
                println!("hahaha, soon");
            }
            false => {
                self.notes.push(ThecaItem {
                    id: self.current_id+1,
                    title: a_title,
                    status: a_status,
                    body: a_body,
                    last_touched: strftime("%FT%T", &now_utc()).ok().unwrap()
                });
            }
        }
        self.current_id += 1;
        println!("added");
    }

    fn delete_item(&mut self, id: int) {
        let remove = self.notes.iter()
            .position(|n| n.id == id)
            .map(|e| self.notes.remove(e))
            .is_some();
        match remove {
            true => {
                println!("removed");
            }
            false => {
                println!("not found");
            }
        }
    }

    // fn edit_item(&mut self) {
    // }

    // fn list_items(&mut self) {
    // }

    // fn search_titles(&mut self, keyword: String) {
    // }

    // fn search_bodies(&mut self, keyword: String) {
    // }

    // fn search_titles_regex(&mut self, regex: String) {
    // }

    // fn search_bodies_regex(&mut self, regex: String) {
    // }
}

fn build_profile(args: &docopt::ArgvMap) -> Result<ThecaProfile, String> {
    if args.get_bool("new_profile") {
        Ok(ThecaProfile {
            current_id: 0,
            encrypted: args.get_bool("--encrypted"),
            notes: vec![]
        })
    } else {
        // set profile folder
        let mut profile_path = if args.get_bool("--profiles-folder") {
            Path::new(args.get_str("PROFILEPATH"))
        } else {
            match os::homedir() {
                Some(ref p) => p.join(".theca"),
                None => Path::new(".").join(".theca")
            }
        };

        // set profile name
        if args.get_bool("-p") {
            profile_path.push(args.get_str("-p").to_string() + ".json");
        } else {
            profile_path.push("default".to_string() + ".json");
        }

        // attempt to read profile
        match profile_path.is_file() {
            false => {
                if profile_path.exists() {
                    Err(format!("{} is not a file.", profile_path.display()))
                } else {
                    Err(format!("{} does not exist.", profile_path.display()))
                }
            }
            true => {
                let mut file = match File::open(&profile_path) {
                    Ok(t) => t,
                    Err(e) => panic!("{}", e.desc)
                };
                let contents = match file.read_to_string() {
                    Ok(t) => t,
                    Err(e) => panic!("{}", e.desc)
                };
                let decoded: ThecaProfile = match json::decode(contents.as_slice()) {
                    Ok(s) => s,
                    Err(e) => panic!("Invalid JSON in {}. {}", profile_path.display(), e)
                };
                Ok(decoded)
            }
        }
    }
}

fn main() {
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.parse())
                      .unwrap_or_else(|e| e.exit());

    println!("{}", args);

    // Setup a ThecaProfile struct
    let mut profile = match build_profile(&args) {
        Ok(p) => p,
        Err(e) => panic!("{}", e)
    };

    // check for all other commands and work on profile
    if args.get_bool("view") {

    } else if args.get_bool("add") {

    } else if args.get_bool("edit") {

    } else if args.get_bool("del") {

    }

    // save altered profile back to disk
    // profile.add_item("woo".to_string(), STARTED.to_string(), "this is the body".to_string());
    profile.save_to_file(&args);

    // if args.get_bool("new_profile") {
    //     // build a new profile
    //     let mut profile = build_profile(args);
    //     println!("profile: {}", json::encode(&profile));
    // } else {
    //     // read in either default.json or PROFILE.json (from args) from .thecaprofiles/

    // }

    // some setup function should do check for existing profile / run preceding line to build a new profile from args etcetcetc..,
    // let mut profile = build_profile(false);

    // profile.add_item("another woo".to_string(), NOSTATUS.to_string(), "".to_string());
    // profile.delete_item(2);
    // profile.delete_item(3);
    // profile.add_item("another woo".to_string(), URGENT.to_string(), "".to_string());

    // println!("profile: {}", json::encode(&profile));
}
