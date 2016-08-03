#![cfg_attr(feature = "unstable", allow(unstable_features))]
#![cfg_attr(feature = "unstable", feature(plugin))]
#![cfg_attr(feature = "unstable", plugin(clippy))]
//  _   _
// | |_| |__   ___  ___ __ _
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// licensed under the MIT license <http://opensource.org/licenses/MIT>
//
// lib.rs
//   main theca struct defintions and command parsing functions.

//! Definitions of Item and Profile and their implementations

extern crate core;
extern crate libc;
extern crate time;
extern crate docopt;
extern crate rustc_serialize;
extern crate regex;
extern crate crypto;
extern crate term;
extern crate rand;
extern crate tempdir;

// std lib imports
use std::fmt;
use std::env;
use std::io::{self, stdin, Read, Write};
use std::iter::repeat;
use std::fs::{File, create_dir};
use std::default::Default;

// random things
use regex::Regex;
use rustc_serialize::{Decodable, Encodable};
use rustc_serialize::json::{decode, as_pretty_json, Encoder};
use time::{now, strftime};

// theca imports
use lineformat::LineFormat;
use utils::c::istty;
use utils::{drop_to_editor, pretty_line, format_field, get_yn_input, sorted_print,
            localize_last_touched_string, parse_last_touched, find_profile_folder, get_password,
            profiles_in_folder, profile_fingerprint, extract_status};
use errors::{Result, Error};
use crypt::{encrypt, decrypt, password_to_key};

pub use self::libc::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};

#[macro_use]pub mod errors;
pub mod lineformat;
pub mod utils;
pub mod crypt;

/// Current version of theca
pub fn version() -> String {
    format!("theca {}", option_env!("THECA_BUILD_VER").unwrap_or(""))
}

/// theca docopt argument struct
#[derive(RustcDecodable, Clone)]
pub struct Args {
    pub cmd_add: bool,
    pub cmd_clear: bool,
    pub cmd_del: bool,
    pub cmd_decrypt_profile: bool,
    pub cmd_edit: bool,
    pub cmd_encrypt_profile: bool,
    pub cmd_import: bool,
    pub cmd_info: bool,
    pub cmd_list_profiles: bool,
    pub cmd_new_profile: bool,
    pub cmd_search: bool,
    pub cmd_transfer: bool,
    pub cmd__: bool,
    pub arg_id: Vec<usize>,
    pub arg_name: Vec<String>,
    pub arg_pattern: String,
    pub arg_title: String,
    pub flag_body: Vec<String>,
    pub flag_condensed: bool,
    pub flag_datesort: bool,
    pub flag_editor: bool,
    pub flag_encrypted: bool,
    pub flag_json: bool,
    pub flag_key: String,
    pub flag_limit: usize,
    pub flag_new_key: String,
    pub flag_none: bool,
    pub flag_profile: String,
    pub flag_profile_folder: String,
    pub flag_regex: bool,
    pub flag_reverse: bool,
    pub flag_search_body: bool,
    pub flag_started: bool,
    pub flag_urgent: bool,
    pub flag_version: bool,
    pub flag_yes: bool,
}

pub struct BoolFlags {
    pub condensed: bool,
    pub datesort: bool,
    pub editor: bool,
    pub encrypted: bool,
    pub json: bool,
    pub regex: bool,
    pub reverse: bool,
    pub search_body: bool,
    pub yes: bool,
}

impl BoolFlags {
    pub fn from_args(args: &Args) -> BoolFlags {
        BoolFlags {
               condensed: args.flag_condensed,
               datesort: args.flag_datesort,
               editor: args.flag_editor,
               encrypted: args.flag_encrypted,
               json: args.flag_json,
               regex: args.flag_regex,
               reverse: args.flag_reverse,
               search_body: args.flag_search_body,
               yes: args.flag_yes,
        }
    }
}

impl Default for BoolFlags {
    fn default() -> BoolFlags {
        BoolFlags {
            condensed: false,
            datesort: false,
            editor: false,
            encrypted: false,
            json: false,
            regex: false,
            reverse: false,
            search_body: false,
            yes: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    Blank,
    Started,
    Urgent,
}

impl Encodable for Status {
    fn encode<S: rustc_serialize::Encoder>(&self, encoder: &mut S) -> ::std::result::Result<(), S::Error> {
        match *self {
            Status::Blank => {
                encoder.emit_enum("Status", |encoder| {
                             encoder.emit_enum_variant(
                                     "",
                                     0usize,
                                     0usize,
                                     |_| Ok(())
                            )
                })
            }
            Status::Started => {
                encoder.emit_enum("Status", |encoder| {
                             encoder.emit_enum_variant(
                                     "Started",
                                     1usize,
                                     0usize,
                                     |_| Ok(())
                            )
                })
            }
            Status::Urgent => {
                encoder.emit_enum("Status", |encoder| {
                             encoder.emit_enum_variant(
                                     "Urgent",
                                     2usize,
                                     0usize,
                                    |_| Ok(())
                            )
                })
            }
        }
    }
}

impl Decodable for Status {
    fn decode<D: ::rustc_serialize::Decoder>(decoder: &mut D) -> ::std::result::Result<Status, D::Error> {
        decoder.read_enum(
                "Status",
                |decoder| {
                          decoder.read_enum_variant(
                                  &["", "Started", "Urgent"],
                                  |_, i| {
                                        Ok(match i {
                                                0usize => Status::Blank,
                                                1usize => Status::Started,
                                                2usize => Status::Urgent,
                                                _ => panic!("internal error: entered unreachable code"),
                                                                    
                                  }
                        )
                })
        })
    }
}                                                        

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   Status::Blank => "",
                   Status::Started => "Started",
                   Status::Urgent => "Urgent",
               })
    }
}

/// datetime formating string
static DATEFMT: &'static str = "%F %T %z";
/// short datetime formating string for printing
static DATEFMT_SHORT: &'static str = "%F %T";

/// Represents a note within a profile
#[derive(RustcDecodable, RustcEncodable, Clone, Debug)]
pub struct Item {
    pub id: usize,
    pub title: String,
    pub status: Status,
    pub body: String,
    pub last_touched: String,
}

impl Item {
    /// print a note as a line
    fn print(&self, line_format: &LineFormat, search_body: bool) -> Result<()> {
        self.write(&mut io::stdout(), line_format, search_body)
    }

    fn write<T: Write>(&self,
                       output: &mut T,
                       line_format: &LineFormat,
                       search_body: bool)
                       -> Result<()> {
        let column_seperator: String = repeat(' ')
                                           .take(line_format.colsep)
                                           .collect();
        try!(write!(output,
                    "{}",
                    format_field(&self.id.to_string(), line_format.id_width, false)));
        try!(write!(output, "{}", column_seperator));
        if !self.body.is_empty() && !search_body {
            try!(write!(output,
                        "{}",
                        format_field(&self.title, line_format.title_width - 4, true)));
            try!(write!(output, "{}", format_field(&" (+)".to_string(), 4, false)));
        } else {
            try!(write!(output,
                        "{}",
                        format_field(&self.title, line_format.title_width, true)));
        }
        try!(write!(output, "{}", column_seperator));
        if line_format.status_width != 0 {
            try!(write!(output,
                        "{}",
                        format_field(&format!("{:?}", self.status), line_format.status_width, false)));
            try!(write!(output, "{}", column_seperator));
        }
        try!(writeln!(output,
                      "{}",
                      format_field(&try!(localize_last_touched_string(&*self.last_touched)),
                                   line_format.touched_width,
                                   false)));
        if search_body {
            for l in self.body.lines() {
                try!(writeln!(output, "\t{}", l));
            }
        }
        Ok(())
    }
}

/// Main container of a theca profile file
#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct Profile {
    pub encrypted: bool,
    pub notes: Vec<Item>,
}

impl Profile {
    fn from_scratch(profile_folder: &str,
                    encrypted: bool,
                    yes: bool)
                    -> Result<(Profile, u64)> {
        let profile_path = try!(find_profile_folder(profile_folder));
        // if the folder doesn't exist, make it yo!
        if !profile_path.exists() {
            if !yes {
                let message = format!("{} doesn't exist, would you like to create it?\n", profile_path.display());
                if !try!(get_yn_input(&message)) {
                    return specific_fail_str!("ok bye ♥");
                }
            }
            try!(create_dir(&profile_path));
        }
        Ok((Profile {
            encrypted: encrypted,
            notes: vec![],
        },
            0u64))
    }

    fn from_existing_profile(profile_name: &str,
                             profile_folder: &str,
                             key: &str,
                             encrypted: bool)
                             -> Result<(Profile, u64)> {
        // set profile folder
        let mut profile_path = try!(find_profile_folder(profile_folder));

        // set profile name
        profile_path.push(&(profile_name.to_string() + ".json"));

        // attempt to read profile
        if profile_path.is_file() {
            let mut file = try!(File::open(&profile_path));
            let mut contents_buf = vec![];
            try!(file.read_to_end(&mut contents_buf));
            let contents = if encrypted {
                let key = password_to_key(&key[..]);
                try!(String::from_utf8(try!(decrypt(&*contents_buf, &*key))))
            } else {
                try!(String::from_utf8(contents_buf))
            };
            let decoded: Profile = match decode(&*contents) {
                Ok(s) => s,
                Err(_) => return specific_fail!(format!("invalid JSON in {}", profile_path.display())),
            };
            let fingerprint = try!(profile_fingerprint(profile_path));
            Ok((decoded, fingerprint))
        } else if profile_path.exists() {
            specific_fail!(format!("{} is not a file.", profile_path.display()))
        } else {
            specific_fail!(format!("{} does not exist.", profile_path.display()))
        }
    }

    /// setup a Profile struct based on the command line arguments
    pub fn new(profile_name: &str,
               profile_folder: &str,
               key: &str,
               new_profile: bool,
               encrypted: bool,
               yes: bool)
               -> Result<(Profile, u64)> {
        if new_profile {
            Profile::from_scratch(profile_folder, encrypted, yes)
        } else {
            Profile::from_existing_profile(profile_name, profile_folder, key, encrypted)
        }
    }

    /// remove all notes from the profile
    pub fn clear(&mut self, yes: bool) -> Result<()> {
        if !yes {
            let message = "are you sure you want to delete all the notes in this profile?\n";
            if !try!(get_yn_input(&message)) {
                return specific_fail_str!("ok bye ♥");
            }
        }
        self.notes.truncate(0);
        Ok(())
    }

    // FIXME (this as well as transfer_note, shouldn't *need* to take all of `args`)
    /// save the profile back to file (either plaintext or encrypted)
    pub fn save_to_file(&mut self, args: &Args, fingerprint: &u64) -> Result<()> {
        // set profile folder
        let mut profile_path = try!(find_profile_folder(&args.flag_profile_folder));

        // set file name
        if args.cmd_new_profile {
            profile_path.push(&(args.arg_name[0].to_string() + ".json"));
        } else {
            profile_path.push(&(args.flag_profile.to_string() + ".json"));
        }

        if args.cmd_new_profile && profile_path.exists() && !args.flag_yes {
            let message = format!("profile {} already exists would you like to overwrite it?\n",
                     profile_path.display());
            if !try!(get_yn_input(&message)) {
                return specific_fail_str!("ok bye ♥");
            }
        }

        if fingerprint > &0u64 {
            let new_fingerprint = try!(profile_fingerprint(&profile_path));
            if &new_fingerprint != fingerprint && !args.flag_yes {
                let message = format!("changes have been made to the profile '{}' on disk since it was \
                          loaded, would you like to attempt to merge them?\n",
                         args.flag_profile);
                if !try!(get_yn_input(&message)) {
                    return specific_fail_str!("ok bye ♥");
                }
                let mut new_args = args.clone();
                if args.flag_editor {
                    new_args.flag_editor = false;
                    new_args.flag_body[0] = match self.notes.last() {
                        Some(n) => n.body.clone(),
                        None => "".to_string(),
                    };
                }
                let (mut changed_profile, changed_fingerprint) = try!(
                    Profile::new(
                        &new_args.flag_profile,
                        &new_args.flag_profile_folder,
                        &new_args.flag_key,
                        new_args.cmd_new_profile,
                        new_args.flag_encrypted,
                        new_args.flag_yes
                    )
                );
                try!(parse_cmds(&mut changed_profile, &mut new_args, &changed_fingerprint));
                try!(changed_profile.save_to_file(&new_args, &0u64));
                return Ok(());
            }
        }

        // open file
        let mut file = try!(File::create(profile_path));

        // encode to buffer
        let mut json_prof = String::new();
        {
            let mut encoder = Encoder::new_pretty(&mut json_prof);
            try!(self.encode(&mut encoder));
        }

        // encrypt json if its an encrypted profile
        let buffer = if self.encrypted {
            let key = password_to_key(&*args.flag_key);
            try!(encrypt(&json_prof.into_bytes(), &*key))
        } else {
            json_prof.into_bytes()
        };

        // write buffer to file
        try!(file.write_all(&buffer));

        Ok(())
    }

    // FIXME (this as well as save_to_file, shouldn't *need* to take all of `args`)
    /// transfer a note from the profile to another profile
    pub fn transfer_note(&mut self, args: &Args) -> Result<()> {
        if args.flag_profile == args.arg_name[0] {
            return specific_fail!(format!("cannot transfer a note from a profile to itself ({} -> {})",
                                   args.flag_profile,
                                   args.arg_name[0]));
        }

        let mut trans_args = args.clone();
        trans_args.flag_profile = args.arg_name[0].clone();
        let (mut trans_profile, trans_fingerprint) = try!(Profile::new(
            &args.arg_name[0],
            &args.flag_profile_folder,
            &args.flag_key,
            args.cmd_new_profile,
            args.flag_encrypted,
            args.flag_yes
        ));

        if self.notes
               .iter()
               .find(|n| n.id == args.arg_id[0])
               .map(|n| trans_profile.add_note(&n.title,
                                          &[n.body.clone()],
                                          Some(n.status),
                                          false,
                                          false,
                                          false))
               .is_some() {
            if self.notes
                   .iter()
                   .position(|n| n.id == args.arg_id[0])
                   .map(|e| self.notes.remove(e))
                   .is_some() {
                try!(trans_profile.save_to_file(&trans_args, &trans_fingerprint))
            } else {
                return specific_fail!(format!("couldn't remove note {} in {}, aborting nothing will be \
                                        saved",
                                       args.arg_id[0],
                                       args.flag_profile))
            }
        } else {
            return specific_fail!(format!("could not transfer note {} from {} -> {}",
                                   args.arg_id[0],
                                   args.flag_profile,
                                   args.arg_name[0]))
        }
        println!("transfered [{}: note {} -> {}: note {}]",
                 args.flag_profile,
                 args.arg_id[0],
                 args.arg_name[0],
                 trans_profile.notes.last().map_or(0, |n| n.id));
        Ok(())
    }

    /// add a item to the profile
    pub fn add_note(&mut self,
                    title: &str,
                    body: &[String],
                    status: Option<Status>,
                    use_stdin: bool,
                    use_editor: bool,
                    print_msg: bool)
                    -> Result<()> {
        let title = title.replace("\n", "").to_string();

        let body = if use_stdin {
            let mut buf = String::new();
            try!(stdin().read_to_string(&mut buf));
            buf.to_owned()
        } else if !use_editor {
            if body.is_empty() {
                "".to_string()
            } else {
                body[0].clone()
            }
        } else if istty(STDOUT_FILENO) && istty(STDIN_FILENO) {
            try!(drop_to_editor(&"".to_string()))
        } else {
            "".to_string()
        };

        let new_id = match self.notes.last() {
            Some(n) => n.id,
            None => 0,
        };
        self.notes.push(Item {
            id: new_id + 1,
            title: title,
            status: status.unwrap_or(Status::Blank),
            body: body,
            last_touched: try!(strftime(DATEFMT, &now())),
        });
        if print_msg {
            println!("note {} added", new_id + 1);
        }
        Ok(())
    }

    /// delete an item from the profile
    pub fn delete_note(&mut self, id: &[usize]) {
        for nid in id.iter() {
            let remove = self.notes
                             .iter()
                             .position(|n| &n.id == nid)
                             .map(|e| self.notes.remove(e))
                             .is_some();
            if remove {
                println!("deleted note {}", nid);
            } else {
                println!("note {} doesn't exist", nid);
            }
        }
    }

    /// edit an item in the profile
    pub fn edit_note(&mut self,
                     id: usize,
                     title: &str,
                     body: &[String],
                     status: Option<Status>,
                     use_stdin: bool,
                     flags: BoolFlags)
                     -> Result<()> {
        // let id = args.arg_id[0];
        let item_pos: usize = match self.notes.iter().position(|n| n.id == id) {
            Some(i) => i,
            None => return specific_fail!(format!("note {} doesn't exist", id)),
        };
        let use_editor = flags.editor;
        let encrypted = flags.encrypted;
        let yes = flags.yes;
        if !title.is_empty() {
            if title.replace("\n", "") == "-" {
                if !use_stdin {
                    let mut buf = String::new();
                    try!(stdin().read_to_string(&mut buf));
                    self.notes[item_pos].body = buf.to_owned();
                } else {
                    self.notes[item_pos].title = title.replace("\n", "")
                                                      .to_string()
                }
            } else {
                self.notes[item_pos].title = title.replace("\n", "")
                                                  .to_string()
            }
            // change title
        }
        self.notes[item_pos].status = status.unwrap_or(Status::Blank);

        if !body.is_empty() || use_editor || use_stdin {
            // change body
            self.notes[item_pos].body = if use_stdin {
                let mut buf = String::new();
                try!(stdin().read_to_string(&mut buf));
                buf.to_owned()
            } else if use_editor {
                if istty(STDOUT_FILENO) && istty(STDIN_FILENO) {
                    if encrypted && !yes {
                        let message = format!("{0}\n\n{1}\n{2}\n\n{0}\n{3}\n",
                                    "## [WARNING] ##",
                                    "continuing will write the body of the decrypted note to a \
                                    temporary",
                                    "file, increasing the possibilty it could be recovered \
                                    later.",
                                    "Are you sure you want to continue?\n");
                        if !try!(get_yn_input(&message)) {
                            return specific_fail_str!("ok bye ♥");
                        }
                    }
                    let new_body = try!(drop_to_editor(&self.notes[item_pos].body));
                    if self.notes[item_pos].body != new_body {
                        new_body
                    } else {
                        self.notes[item_pos].body.clone()
                    }
                } else {
                    self.notes[item_pos].body.clone()
                }
            } else {
                body[0].clone()
            }
        };

        // update last_touched
        self.notes[item_pos].last_touched = try!(strftime(DATEFMT, &now()));
        println!("edited note {}", self.notes[item_pos].id);
        Ok(())
    }

    /// print information about the profile
    pub fn stats(&mut self, name: &str) -> Result<()> {
        let no_s = self.notes.iter().filter(|n| n.status == Status::Blank).count();
        let started_s = self.notes
                            .iter()
                            .filter(|n| n.status == Status::Started)
                            .count();
        let urgent_s = self.notes
                           .iter()
                           .filter(|n| n.status == Status::Urgent)
                           .count();
        let tty = istty(STDOUT_FILENO);
        let min = match self.notes
                            .iter()
                            .min_by_key(|n| match parse_last_touched(&*n.last_touched) {
                                Ok(o) => o,
                                Err(_) => now(),
                            }) {
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => return specific_fail_str!("last_touched is not properly formated"),
        };
        let max = match self.notes
                            .iter()
                            .max_by_key(|n| match parse_last_touched(&*n.last_touched) {
                                Ok(o) => o,
                                Err(_) => now(),
                            }) {
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => return specific_fail_str!("last_touched is not properly formated"),
        };
        try!(pretty_line("name: ", &format!("{}\n", name), tty));
        try!(pretty_line("encrypted: ", &format!("{}\n", self.encrypted), tty));
        try!(pretty_line("notes: ", &format!("{}\n", self.notes.len()), tty));
        try!(pretty_line("statuses: ",
                         &format!("none: {}, started: {}, urgent: {}\n",
                                  no_s,
                                  started_s,
                                  urgent_s),
                         tty));
        try!(pretty_line("note ages: ",
                         &format!("oldest: {}, newest: {}\n", min, max),
                         tty));
        Ok(())
    }

    /// print a full item
    pub fn view_note(&mut self, id: usize, json: bool, condensed: bool) -> Result<()> {
        let id = id;
        let note_pos = match self.notes.iter().position(|n| n.id == id) {
            Some(i) => i,
            None => return specific_fail!(format!("note {} doesn't exist", id)),
        };
        if json {
            println!("{}", as_pretty_json(&self.notes[note_pos].clone()));
        } else {
            let tty = istty(STDOUT_FILENO);

            if condensed {
                try!(pretty_line("id: ", &format!("{}\n", self.notes[note_pos].id), tty));
                try!(pretty_line("title: ", &format!("{}\n", self.notes[note_pos].title), tty));
                if self.notes[note_pos].status != Status::Blank {
                    try!(pretty_line("status: ",
                                     &format!("{}\n", self.notes[note_pos].status),
                                     tty));
                }
                try!(pretty_line("last touched: ",
                                 &format!("{}\n",
                                          try!(
                                localize_last_touched_string(
                                    &*self.notes[note_pos].last_touched
                                )
                            )),
                                 tty));
            } else {
                try!(pretty_line("id\n--\n", &format!("{}\n\n", self.notes[note_pos].id), tty));
                try!(pretty_line("title\n-----\n",
                                 &format!("{}\n\n", self.notes[note_pos].title),
                                 tty));
                if self.notes[note_pos].status != Status::Blank {
                    try!(pretty_line("status\n------\n",
                                     &format!("{:?}\n\n", self.notes[note_pos].status),
                                     tty));
                }
                try!(pretty_line("last touched\n------------\n",
                                 &format!("{}\n\n",
                                          try!(
                                localize_last_touched_string(
                                    &*self.notes[note_pos].last_touched
                                )
                            )),
                                 tty));
            };

            // body
            if !self.notes[note_pos].body.is_empty() {
                if condensed {
                    try!(pretty_line("body: ", &format!("{}\n", self.notes[note_pos].body), tty));
                } else {
                    try!(pretty_line("body\n----\n",
                                     &format!("{}\n\n", self.notes[note_pos].body),
                                     tty));
                };
            }
        }
        Ok(())
    }

    /// print all notes in the profile
    pub fn list_notes(&mut self,
                      limit: usize,
                      flags: BoolFlags,
                      status: Option<Status>)
                      -> Result<()> {
        if !self.notes.is_empty() {
            try!(sorted_print(&mut self.notes.clone(),
                              limit,
                              flags,
                              status));
        } else if flags.json {
            println!("[]");
        } else {
            println!("this profile is empty");
        }
        Ok(())
    }

    /// print notes search for in the profile
    pub fn search_notes(&mut self,
                        pattern: &str,
                        limit: usize,
                        flags: BoolFlags,
                        status: Option<Status>)
                        -> Result<()> {
        let notes: Vec<Item> = if flags.regex {
            let re = match Regex::new(&pattern[..]) {
                Ok(r) => r,
                Err(e) => return specific_fail!(format!("regex error: {}.", e)),
            };
            self.notes
                .iter()
                .filter(|n| if flags.search_body {
                    re.is_match(&*n.body)
                } else {
                    re.is_match(&*n.title)
                })
                .cloned()
                .collect()
        } else {
            self.notes
                .iter()
                .filter(|n| if flags.search_body {
                    n.body.contains(&pattern[..])
                } else {
                    n.title.contains(&pattern[..])
                })
                .cloned()
                .collect()
        };
        if !notes.is_empty() {
            try!(sorted_print(&mut notes.clone(),
                              limit,
                              flags,
                              status));
        } else if flags.json {
            println!("[]");
        } else {
            println!("nothing found");
        }
        Ok(())
    }
}

pub fn setup_args(args: &mut Args) -> Result<()> {
    if let Ok(val) = env::var("THECA_DEFAULT_PROFILE") {
        if args.flag_profile.is_empty() && !val.is_empty() {
            args.flag_profile = val;
        }
    }

    if let Ok(val) = env::var("THECA_PROFILE_FOLDER") {
        if args.flag_profile_folder.is_empty() && !val.is_empty() {
            args.flag_profile_folder = val;
        }
    }

    // if key is provided but --encrypted not set, it prob should be
    if !args.flag_key.is_empty() && !args.flag_encrypted {
        args.flag_encrypted = true;
    }

    // if profile is encrypted try to set the key
    if args.flag_encrypted && args.flag_key.is_empty() {
        args.flag_key = try!(get_password());
    }

    // if no profile is provided via cmd line or env set it to default
    if args.flag_profile.is_empty() {
        args.flag_profile = "default".to_string();
    }
    

    Ok(())
}

pub fn parse_cmds(profile: &mut Profile,
                  args: &mut Args,
                  profile_fingerprint: &u64)
                  -> Result<()> {
    let status = try!(extract_status(args.flag_none, args.flag_started, args.flag_urgent));
    let flags = BoolFlags::from_args(args);

    if [args.cmd_add,
        args.cmd_edit,
        args.cmd_encrypt_profile,
        args.cmd_del,
        args.cmd_decrypt_profile,
        args.cmd_transfer,
        args.cmd_clear,
        args.cmd_new_profile]
           .iter()
           .any(|c| c == &true) {
        // add
        if args.cmd_add {
            try!(profile.add_note(&args.arg_title,
                                  &args.flag_body,
                                  status,
                                  args.cmd__,
                                  args.flag_editor,
                                  true));
        }

        // edit
        if args.cmd_edit {
            try!(profile.edit_note(args.arg_id[0],
                                   &args.arg_title,
                                   &args.flag_body,
                                   status,
                                   args.cmd__,
                                   flags));
        }

        // delete
        if args.cmd_del {
            profile.delete_note(&args.arg_id);
        }

        // transfer
        if args.cmd_transfer {
            // transfer a note
            try!(profile.transfer_note(args));
        }

        // clear
        if args.cmd_clear {
            try!(profile.clear(args.flag_yes));
        }

        // decrypt profile
        // FIXME: should test how this interacts with save_to_file when the profile has
        //        changed during execution
        if args.cmd_decrypt_profile {
            profile.encrypted = false; // is it that easy? i think it is
            println!("decrypting '{}'", args.flag_profile);
        }

        // encrypt profile
        // FIXME: should test how this interacts with save_to_file when the profile has
        //        changed during execution
        if args.cmd_encrypt_profile {
            // get the new key
            if args.flag_new_key.is_empty() {
                args.flag_new_key = try!(get_password());
            }

            // set args.key and args.encrypted
            args.flag_encrypted = true;
            args.flag_key = args.flag_new_key.clone();

            // set profile to encrypted
            profile.encrypted = true;
            println!("encrypting '{}'", args.flag_profile);
        }

        // new profile
        if args.cmd_new_profile {
            if args.cmd_new_profile && args.arg_name.is_empty() {
                args.arg_name.push("default".to_string())
            }
            println!("creating profile '{}'", args.arg_name[0]);
        }

        try!(profile.save_to_file(args, profile_fingerprint));
    } else if !args.arg_id.is_empty() {
        try!(profile.view_note(args.arg_id[0], args.flag_json, args.flag_condensed));
    } else if args.cmd_search {
        try!(profile.search_notes(&args.arg_pattern,
                                    args.flag_limit,
                                    flags,
                                    status));
    } else if args.cmd_info {
        try!(profile.stats(&args.flag_profile));
    } else if args.cmd_import {
        // reverse(?) transfer a note
        let mut from_args = args.clone();
        from_args.cmd_transfer = args.cmd_import;
        from_args.cmd_import = false;
        from_args.flag_profile = args.arg_name[0].clone();
        from_args.arg_name[0] = args.flag_profile.clone();

        let (mut from_profile, from_fingerprint) = try!(Profile::new(
                &from_args.flag_profile,
                &from_args.flag_profile_folder,
                &from_args.flag_key,
                from_args.cmd_new_profile,
                from_args.flag_encrypted,
                from_args.flag_yes
            ));

        try!(parse_cmds(&mut from_profile, &mut from_args, &from_fingerprint));
    } else if args.cmd_list_profiles {
        let profile_path = try!(find_profile_folder(&args.flag_profile_folder));
        try!(profiles_in_folder(&profile_path));
    } else if args.arg_id.is_empty() {
        try!(profile.list_notes(args.flag_limit,
                                flags,
                                status));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
#![allow(non_snake_case)]
    use super::{Status, Item};
    use super::lineformat::LineFormat;

    fn write_item_test_case(item: Item, search: bool) -> String {
        let mut bytes: Vec<u8> = vec![];
        let line_format = LineFormat::new(&[item.clone()], false, false).unwrap();
        item.write(&mut bytes, &line_format, search).expect("item.write failed");
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[test]
    fn test_write_item__no_search_non_empty_body() {
        let item = Item {
            id: 0,
            title: "This is a title".into(),
            status: Status::Blank,
            body: "This is the body".into(),
            last_touched: "2016-07-08 15:31:14 -0800".into(),
        };
        assert_eq!(write_item_test_case(item, false),
                   "0   This is a title (+)  2016-07-08 16:31:14\n");
    }

    #[test]
    fn test_write_item__no_search_empty_body() {
        // no search && empty body
        let item = Item {
            id: 0,
            title: "This is a title".into(),
            status: Status::Blank,
            body: "".into(),
            last_touched: "2016-07-08 15:31:14 -0800".into(),
        };
        assert_eq!(write_item_test_case(item, false),
                   "0   This is a title  2016-07-08 16:31:14\n");
    }

    #[test]
    fn test_write_item__search_non_empty_body() {
        let item = Item {
            id: 0,
            title: "This is a title".into(),
            status: Status::Blank,
            body: "This is the body\nit has multiple lines".into(),
            last_touched: "2016-07-08 15:31:14 -0800".into(),
        };
        assert_eq!(write_item_test_case(item, true),
                   "0   This is a title      2016-07-08 16:31:14\n\tThis is the body\n\tit has \
                    multiple lines\n");
    }

    #[test]
    fn test_write_item__search_empty_body() {
        // search && empty body
        let item = Item {
            id: 0,
            title: "This is a title".into(),
            status: Status::Blank,
            body: "".into(),
            last_touched: "2016-07-08 15:31:14 -0800".into(),
        };
        assert_eq!(write_item_test_case(item, true),
                   "0   This is a title  2016-07-08 16:31:14\n");
    }

    #[test]
    fn test_write_item__non_zero_status_width() {
        let item = Item {
            id: 0,
            title: "This is a title".into(),
            status: Status::Started,
            body: "This is the body".into(),
            last_touched: "2016-07-08 15:31:14 -0800".into(),
        };
        assert_eq!(write_item_test_case(item, false),
                   "0   This is a title (+)  Started  2016-07-08 16:31:14\n");

    }
}
