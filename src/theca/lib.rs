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

#![crate_name="theca"]
#![crate_type="lib"]
#![feature(core)]
#![feature(libc)]
#![feature(collections)]
#![feature(old_io)]
#![feature(rustc_private)]
#![feature(os)]
#![feature(io)]
#![feature(fs_time)]
#![feature(path)]
#![feature(path_ext)]


//! Definitions of ThecaItem and ThecaProfile and their implementations

extern crate core;
extern crate libc;
extern crate time;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
extern crate regex;
extern crate crypto;
extern crate term;
extern crate rand;
extern crate tempdir;

// std lib imports
use std::env::{var};
use std::old_io::{stdin};
use std::io::{Read, Write};
use std::iter::{repeat};
use std::path::Path;
use std::fs::{File, PathExt, create_dir};

// random things
use regex::{Regex};
use rustc_serialize::{Encodable};
use rustc_serialize::json::{decode, as_pretty_json, Encoder};
use time::{now, strftime};

// theca imports
use lineformat::{LineFormat};
use utils::c::{istty};
use utils::{drop_to_editor, pretty_line, format_field,
            get_yn_input, sorted_print, localize_last_touched_string,
            parse_last_touched, find_profile_folder, get_password,
            profiles_in_folder};
use errors::{ThecaError, GenericError};
use crypt::{encrypt, decrypt, password_to_key};

pub use self::libc::{
    STDIN_FILENO,
    STDOUT_FILENO,
    STDERR_FILENO
};

#[macro_use] pub mod errors;
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
    pub flag_yes: bool
}

/// No status text
static NOSTATUS: &'static str = "";
/// Started status text
static STARTED: &'static str = "Started";
/// Urgent status text
static URGENT: &'static str = "Urgent";

/// datetime formating string
static DATEFMT: &'static str = "%F %T %z";
/// short datetime formating string for printing
static DATEFMT_SHORT: &'static str = "%F %T";

/// Represents a note within a profile
#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct ThecaItem {
    pub id: usize,
    pub title: String,
    pub status: String,
    pub body: String,
    pub last_touched: String
}

impl ThecaItem {
    /// print a note as a line
    fn print(
        &self,
        line_format: &LineFormat,
        search_body: bool
    ) -> Result<(), ThecaError> {
        let column_seperator: String = repeat(' ').take(line_format.colsep)
                                                  .collect();
        print!("{}", format_field(
            &self.id.to_string(),
            line_format.id_width,
            false
        ));
        print!("{}", column_seperator);
        match !self.body.is_empty() && !search_body {
            true => {
                print!("{}", format_field(
                    &self.title,
                    line_format.title_width-4,
                    true
                ));
                print!("{}", format_field(&" (+)".to_string(), 4, false));
            },
            false => {
                print!("{}", format_field(
                    &self.title,
                    line_format.title_width,
                    true
                ));
            }
        }
        print!("{}", column_seperator);
        if line_format.status_width != 0 {
            print!("{}", format_field(
                &self.status,
                line_format.status_width,
                false
            ));
            print!("{}", column_seperator);
        }
        print!("{}", format_field(
            &try!(localize_last_touched_string(&*self.last_touched)),
            line_format.touched_width,
            false
        ));
        print!("\n");
        if search_body {
            for l in self.body.lines() {
                println!("\t{}", l);
            }
        }
        Ok(())
    }
}

/// Main container of a theca profile file
#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct ThecaProfile {
    pub encrypted: bool,
    pub notes: Vec<ThecaItem>
}

impl ThecaProfile {
    /// setup a ThecaProfile struct based on the command line arguments
    pub fn new(
        profile_name: &String,
        profile_folder: &String,
        key: &String,
        new_profile: bool,
        encrypted: bool,
        yes: bool
    ) -> Result<(ThecaProfile, u64), ThecaError> {
        if new_profile {
            let profile_pathbuf = try!(find_profile_folder(profile_folder));
            let profile_path: &Path = &profile_pathbuf;
            // if the folder doesn't exist, make it yo!
            if !profile_path.exists() {
                if !yes {
                    println!(
                        "{} doesn't exist, would you like to create it?",
                        profile_path.display()
                    );
                    if !try!(get_yn_input()) {
                        specific_fail_str!("ok bye ♥");
                    }
                }
                try!(create_dir(&profile_path)); // FIXME: bridging old<->new path stuff...
            }
            Ok((ThecaProfile {
                encrypted: encrypted,
                notes: vec![]
            }, 0u64))
        } else {
            // set profile folder
            let mut profile_pathbuf = try!(find_profile_folder(profile_folder));

            // set profile name
            profile_pathbuf.push(&(profile_name.to_string() + ".json"));
            let profile_path: &Path = &profile_pathbuf;
            
            // attempt to read profile
            match profile_path.is_file() {
                false => {
                    if profile_path.exists() {
                        specific_fail!(format!(
                            "{} is not a file.",
                            profile_path.display()
                        ));
                    } else {
                        // FIXME
                        // if profile_name == &"default".to_string() {
                        //     println!(
                        //         "{} does not exist, would you like to create it? (this will be the default profile)",
                        //         profile_path.display()
                        //     );
                        //     match try!(get_yn_input()) {
                        //         true => Ok((
                        //             ThecaProfile {
                        //                 encrypted: encrypted,
                        //                 notes: vec![]
                        //             }, 
                        //             0u64
                        //         )),
                        //         false => specific_fail_str!("ok bye ♥")
                        //     }
                        // } else {
                        //     specific_fail!(format!(
                        //         "{} does not exist.",
                        //         profile_path.display()
                        //     ));
                        // }
                        specific_fail!(format!(
                            "{} does not exist.",
                            profile_path.display()
                        ));
                    }
                }
                true => {
                    let mut file = try!(File::open(profile_path));
                    let mut contents_buf: Vec<u8> = vec![];
                    try!(file.read_to_end(&mut contents_buf));
                    let contents = match encrypted {
                        false => try!(String::from_utf8(contents_buf)),
                        true => {
                            let key = password_to_key(&key[..]);
                                try!(String::from_utf8(try!(decrypt(
                                    &*contents_buf,
                                    &*key
                                ))))
                        }
                    };
                    let decoded: ThecaProfile = match decode(&*contents) {
                        Ok(s) => s,
                        Err(_) => specific_fail!(format!(
                            "invalid JSON in {}",
                            profile_path.display()
                        ))
                    };
                    Ok((decoded, try!(profile_path.metadata()).modified()))
                }
            }
        }
    }

    /// remove all notes from the profile
    pub fn clear(&mut self, yes: bool) -> Result<(), ThecaError> {
        if !yes {
            println!(
                "are you sure you want to delete all the notes in this profile?"
            );
            if !try!(get_yn_input()) {specific_fail_str!("ok bye ♥");}
        }
        self.notes.truncate(0);
        Ok(())
    }

    // FIXME (this as well as transfer_note, shouldn't *need* to take all of `args`)
    /// save the profile back to file (either plaintext or encrypted)
    pub fn save_to_file(
        &mut self,
        args: &Args,
        fingerprint: &u64
    ) -> Result<(), ThecaError> {
        // set profile folder
        let mut profile_pathbuf = try!(
            find_profile_folder(&args.flag_profile_folder)
        );

        // set file name
        match args.cmd_new_profile {
            true => profile_pathbuf.push(&(args.arg_name[0].to_string() + ".json")),
            false => profile_pathbuf.push(&(args.flag_profile.to_string() + ".json"))
        };

        let profile_path: &Path = &profile_pathbuf;

        if args.cmd_new_profile && profile_path.exists() && !args.flag_yes {
            println!(
                "profile {} already exists would you like to overwrite it?",
                profile_path.display()
            );
            if !try!(get_yn_input()) {
                specific_fail_str!("ok bye ♥");
            }
        }

        if fingerprint > &0u64 {
            let new_fingerprint = try!(profile_path.metadata()).modified();
            if &new_fingerprint != fingerprint && !args.flag_yes {
                println!(
                    "changes have been made to the profile '{}' on disk since it was loaded, would you like to attempt to merge them?",
                    args.flag_profile
                );
                if !try!(get_yn_input()) {
                    specific_fail_str!("ok bye ♥");
                }
                let mut new_args = args.clone();
                if args.flag_editor { 
                    new_args.flag_editor = false;
                    new_args.flag_body[0] = match self.notes.last() {
                        Some(n) => n.body.clone(),
                        None => "".to_string()
                    };
                }
                let (mut changed_profile, changed_fingerprint) = try!(
                    ThecaProfile::new(
                        &new_args.flag_profile,
                        &new_args.flag_profile_folder,
                        &new_args.flag_key,
                        new_args.cmd_new_profile,
                        new_args.flag_encrypted,
                        new_args.flag_yes
                    )
                );
                try!(parse_cmds(
                    &mut changed_profile,
                    &mut new_args,
                    &changed_fingerprint
                ));
                try!(changed_profile.save_to_file(&new_args, &0u64));
                return Ok(())
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
        let buffer = match self.encrypted {
            true => {
                let key = password_to_key(&*args.flag_key);
                try!(encrypt(
                    &json_prof.into_bytes(),
                    &*key
                ))
            },
            false => json_prof.into_bytes()
        };

        // write buffer to file
        try!(file.write_all(&buffer));

        Ok(())
    }

    // FIXME (this as well as save_to_file, shouldn't *need* to take all of `args`)
    /// transfer a note from the profile to another profile
    pub fn transfer_note(&mut self, args: &Args) -> Result<(), ThecaError> {
        if args.flag_profile == args.arg_name[0] {
            specific_fail!(format!(
                "cannot transfer a note from a profile to itself ({} -> {})",
                args.flag_profile,
                args.arg_name[0]
            ));
        }

        let mut trans_args = args.clone();
        trans_args.flag_profile = args.arg_name[0].clone();
        let (mut trans_profile, trans_fingerprint) = try!(ThecaProfile::new(
            &args.arg_name[0],
            &args.flag_profile_folder,
            &args.flag_key,
            args.cmd_new_profile,
            args.flag_encrypted,
            args.flag_yes
        ));

        match self.notes.iter().find(|n| n.id == args.arg_id[0])
                        .map(|n| {
                            let (started, urgent) = (
                                n.status == STARTED,
                                n.status == URGENT
                            );
                            trans_profile.add_note(
                                &n.title,
                                &vec![n.body.clone()],
                                started,
                                urgent,
                                false,
                                false,
                                false
                            )
                        }).is_some() {
            true =>  {
                match self.notes.iter().position(|n| n.id == args.arg_id[0])
                                   .map(|e| self.notes.remove(e)).is_some() {
                    true => try!(trans_profile.save_to_file(
                        &trans_args,
                        &trans_fingerprint
                    )),
                    false => specific_fail!(format!(
                        "couldn't remove note {} in {}, aborting nothing will be saved",
                        args.arg_id[0],
                        args.flag_profile
                    ))
                };
            },
            false => specific_fail!(format!(
                "could not transfer note {} from {} -> {}",
                args.arg_id[0],
                args.flag_profile,
                args.arg_name[0]
            ))
        };
        println!(
            "transfered [{}: note {} -> {}: note {}]",
            args.flag_profile,
            args.arg_id[0],
            args.arg_name[0],
            match trans_profile.notes.last() {
                Some(n) => n.id,
                None => 0
            }
        );
        Ok(())
    }

    /// add a item to the profile
    pub fn add_note(
        &mut self,
        title: &String,
        body: &Vec<String>,
        started: bool,
        urgent: bool,
        use_stdin: bool,
        use_editor: bool,
        print_msg: bool
    ) -> Result<(), ThecaError> {
        let title = title.replace("\n", "").to_string();
        let status = if started {
            STARTED.to_string()
        } else if urgent {
            URGENT.to_string()
        } else {
            NOSTATUS.to_string()
        };

        let body = match use_stdin {
            false => match use_editor {
                false => match body.is_empty() {
                    true => "".to_string(),
                    false => body[0].clone(),
                },
                true => {
                    match istty(STDOUT_FILENO) && istty(STDIN_FILENO) {
                        true => try!(drop_to_editor(&"".to_string())),
                        false => "".to_string()
                    }
                }
            },
            true => { try!(stdin().read_to_string()) }
        };

        let new_id = match self.notes.last() {
            Some(n) => n.id,
            None => 0
        };
        self.notes.push(ThecaItem {
            id: new_id + 1,
            title: title,
            status: status,
            body: body,
            last_touched: try!(strftime(DATEFMT, &now()))
        });
        if print_msg { println!("note {} added", new_id+1); }
        Ok(())
    }

    /// delete an item from the profile
    pub fn delete_note(&mut self, id: &Vec<usize>) {
        for nid in id.iter() {
            let remove = self.notes.iter()
                .position(|n| &n.id == nid)
                .map(|e| self.notes.remove(e))
                .is_some();
            match remove {
                true => {
                    println!("deleted note {}", nid);
                }
                false => {
                    println!("note {} doesn't exist", nid);
                }
            }
        }
    }

    /// edit an item in the profile
    pub fn edit_note(
        &mut self,
        id: usize,
        title: &String,
        body: &Vec<String>,
        started: bool,
        urgent: bool,
        no_status: bool,
        use_stdin: bool,
        use_editor: bool,
        encrypted: bool,
        yes: bool
    ) -> Result<(), ThecaError> {
        // let id = args.arg_id[0];
        let item_pos: usize = match self.notes.iter().position(|n| n.id == id) {
            Some(i) => i,
            None => specific_fail!(format!("note {} doesn't exist", id))
        };
        if !title.is_empty() {
            match title.replace("\n", "") == "-" {
                true => match !use_stdin {
                    true => self.notes[item_pos].body = try!(
                        stdin().read_to_string()
                    ),
                    false => self.notes[item_pos].title = title.replace("\n", "")
                                                               .to_string()
                },
                false => self.notes[item_pos].title = title.replace("\n", "")
                                                           .to_string()
            }
            // change title
        }
        if started || urgent || no_status {
            // change status
            if started {
                self.notes[item_pos].status = STARTED.to_string();
            } else if urgent {
                self.notes[item_pos].status = URGENT.to_string();
            } else if no_status {
                self.notes[item_pos].status = NOSTATUS.to_string();
            }
        }

        if !body.is_empty() || use_editor || use_stdin {
            // change body
            self.notes[item_pos].body = match use_stdin {
                true => try!(stdin().read_to_string()),
                false => match use_editor {
                    true => {
                        match istty(STDOUT_FILENO) && istty(STDIN_FILENO) {
                            true => {
                                if encrypted && !yes {
                                    println!(
                                        "{0}\n\n{1}\n{2}\n\n{0}\n{3}\n",
                                        "## [WARNING] ##",
                                        "continuing will write the body of the decrypted note to a temporary",
                                        "file, increasing the possibilty it could be recovered later.",
                                        "Are you sure you want to continue?"
                                    );
                                    if !try!(get_yn_input()) {specific_fail_str!("ok bye ♥");}
                                }
                                let new_body = try!(
                                    drop_to_editor(&self.notes[item_pos].body)
                                );
                                match self.notes[item_pos].body != new_body {
                                    true => new_body,
                                    false => self.notes[item_pos].body.clone()
                                }     
                            },
                            false => self.notes[item_pos].body.clone()
                        }
                    },
                    false => body[0].clone()
                }
            };
        }

        // update last_touched
        self.notes[item_pos].last_touched = try!(strftime(DATEFMT, &now()));
        println!("edited note {}", self.notes[item_pos].id);
        Ok(())
    }

    /// print information about the profile
    pub fn stats(&mut self, name: &String) -> Result<(), ThecaError> {
        let no_s = self.notes.iter().filter(|n| n.status == "").count();
        let started_s = self.notes.iter().filter(|n| n.status == "Started")
                                         .count();
        let urgent_s = self.notes.iter().filter(|n| n.status == "Urgent")
                                        .count();
        let tty = istty(STDOUT_FILENO);
        let min = match self.notes.iter()
                            .min_by(|n| match parse_last_touched(&*n.last_touched) {
            Ok(o) => o,
            Err(_) => now() // FIXME
        }) {
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => specific_fail_str!(
                "last_touched is not properly formated"
            )
        };
        let max = match self.notes.iter()
                            .max_by(|n| match parse_last_touched(&*n.last_touched) {
            Ok(o) => o,
            Err(_) => now() // FIXME
        }) {
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => specific_fail_str!(
                "last_touched is not properly formated"
            )
        };
        try!(pretty_line("name: ", &format!("{}\n", name), tty));
        try!(pretty_line("encrypted: ", &format!("{}\n", self.encrypted), tty));
        try!(pretty_line("notes: ", &format!("{}\n", self.notes.len()), tty));
        try!(pretty_line("statuses: ", &format!(
            "none: {}, started: {}, urgent: {}\n",
            no_s,
            started_s,
            urgent_s
        ), tty));
        try!(pretty_line("note ages: ", &format!(
            "oldest: {}, newest: {}\n",
            min,
            max
        ), tty));
        Ok(())
    }

    /// print a full item
    pub fn view_note(
        &mut self,
        id: usize,
        json: bool,
        condensed: bool
    ) -> Result<(), ThecaError> {
        let id = id;
        let note_pos = match self.notes.iter().position(|n| n.id == id) {
            Some(i) => i,
            None => specific_fail!(format!("note {} doesn't exist", id))
        };
        match json {
            false => {
                let tty = istty(STDOUT_FILENO);
        
                match condensed {
                    true => {
                        try!(pretty_line("id: ", &format!(
                            "{}\n",
                            self.notes[note_pos].id),
                            tty
                        ));
                        try!(pretty_line("title: ", &format!(
                            "{}\n",
                            self.notes[note_pos].title),
                            tty
                        ));
                        if !self.notes[note_pos].status.is_empty() {
                            try!(pretty_line("status: ", &format!(
                                "{}\n",
                                self.notes[note_pos].status),
                                tty
                            ));
                        }
                        try!(pretty_line(
                            "last touched: ",
                            &format!(
                                "{}\n",
                                try!(
                                    localize_last_touched_string(
                                        &*self.notes[note_pos].last_touched
                                    )
                                )
                            ),
                            tty
                        ));
                    },
                    false => {
                        try!(pretty_line("id\n--\n", &format!(
                            "{}\n\n",
                            self.notes[note_pos].id),
                            tty
                        ));
                        try!(pretty_line("title\n-----\n", &format!(
                            "{}\n\n",
                            self.notes[note_pos].title),
                            tty
                        ));
                        if !self.notes[note_pos].status.is_empty() {
                            try!(pretty_line(
                                "status\n------\n",
                                &format!("{}\n\n", self.notes[note_pos].status),
                                tty
                            ));
                        }
                        try!(pretty_line(
                            "last touched\n------------\n",
                            &format!(
                                "{}\n\n",
                                try!(
                                    localize_last_touched_string(
                                        &*self.notes[note_pos].last_touched
                                    )
                                )
                            ),
                            tty
                        ));
                    }
                };
        
                // body
                if !self.notes[note_pos].body.is_empty() {
                    match condensed {
                        true => {
                            try!(pretty_line("body: ", &format!(
                                "{}\n",
                                self.notes[note_pos].body),
                                tty
                            ));
                        },
                        false => {
                            try!(pretty_line("body\n----\n", &format!(
                                "{}\n\n",
                                self.notes[note_pos].body),
                                tty
                            ));
                        }
                    };
                }
            },
            true => println!(
                "{}",
                as_pretty_json(&self.notes[note_pos].clone())
            )
        }
        Ok(())
    }

    /// print all notes in the profile
    pub fn list_notes(
        &mut self,
        limit: usize,
        condensed: bool,
        json: bool,
        datesort: bool,
        reverse: bool,
        search_body: bool
    ) -> Result<(), ThecaError> {
        if self.notes.len() > 0 {
            try!(sorted_print(
                &mut self.notes.clone(),
                limit,
                condensed,
                json,
                datesort,
                reverse,
                search_body
            ));
        } else {
            match json {
                true => println!("[]"),
                false => println!("this profile is empty")
            }
        }
        Ok(())
    }

    /// print notes search for in the profile
    pub fn search_notes(
        &mut self,
        pattern: &String,
        regex: bool,
        limit: usize,
        condensed: bool,
        json: bool,
        datesort: bool,
        reverse: bool,
        search_body: bool
    ) -> Result<(), ThecaError> {
        let notes: Vec<ThecaItem> = match regex {
            true => {
                let re = match Regex::new(&pattern[..]) {
                    Ok(r) => r,
                    Err(e) => specific_fail!(
                        format!("regex error: {}.", e.msg)
                    )
                };
                self.notes.iter().filter(|n| match search_body {
                    true => re.is_match(&*n.body),
                    false => re.is_match(&*n.title)
                }).map(|n| n.clone()).collect()
            },
            false => {
                self.notes.iter().filter(|n| match search_body {
                    true => n.body.contains(&pattern[..]),
                    false => n.title.contains(&pattern[..])
                }).map(|n| n.clone()).collect()
            }
        };
        if notes.len() > 0 {
            try!(sorted_print(
                &mut notes.clone(),
                limit,
                condensed,
                json,
                datesort,
                reverse,
                search_body
            ));
        } else {
            match json {
                true => println!("[]"),
                false => println!("nothing found")
            }
        }
        Ok(())
    }
}

pub fn setup_args(args: &mut Args) -> Result<(), ThecaError> {
    match var("THECA_DEFAULT_PROFILE") {
        Ok(val) => {
            if args.flag_profile.is_empty() && !val.is_empty() {
                args.flag_profile = val;
            }
        },
        Err(_) => ()
    };

    match var("THECA_PROFILE_FOLDER") {
        Ok(val) => {
            if args.flag_profile_folder.is_empty() && !val.is_empty() {
                args.flag_profile_folder = val;
            }
        },
        Err(_) => ()
    };

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

pub fn parse_cmds(
    profile: &mut ThecaProfile,
    args: &mut Args,
    profile_fingerprint: &u64
) -> Result<(), ThecaError> {
    match [
        args.cmd_add,
        args.cmd_edit,
        args.cmd_encrypt_profile,
        args.cmd_del,
        args.cmd_decrypt_profile,
        args.cmd_transfer,
        args.cmd_clear,
        args.cmd_new_profile
    ].iter().any(|c| c == &true) {
        true => {
            // add
            if args.cmd_add {
                try!(profile.add_note(
                    &args.arg_title,
                    &args.flag_body,
                    args.flag_started,
                    args.flag_urgent,
                    args.cmd__,
                    args.flag_editor,
                    true
                ));
            }

            // edit    
            if args.cmd_edit {
                try!(profile.edit_note(
                    args.arg_id[0],
                    &args.arg_title,
                    &args.flag_body,
                    args.flag_started,
                    args.flag_urgent,
                    args.flag_none,
                    args.cmd__,
                    args.flag_editor,
                    args.flag_encrypted,
                    args.flag_yes
                ));
            }
            
            // delete    
            if args.cmd_del { profile.delete_note(&args.arg_id); }

            // transfer
            if args.cmd_transfer {
                // transfer a note
                try!(profile.transfer_note(args));
            }

            // clear
            if args.cmd_clear { try!(profile.clear(args.flag_yes)); }

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
        },
        false => {
            // view
            if !args.arg_id.is_empty() {
                try!(profile.view_note(
                    args.arg_id[0],
                    args.flag_json,
                    args.flag_condensed
                ));
                return Ok(())
            }

            // search
            if args.cmd_search {
                try!(profile.search_notes(
                    &args.arg_pattern,
                    args.flag_regex,
                    args.flag_limit,
                    args.flag_condensed,
                    args.flag_json,
                    args.flag_datesort,
                    args.flag_reverse,
                    args.flag_search_body
                ));
                return Ok(())
            }

            // stats
            if args.cmd_info {
                try!(profile.stats(&args.flag_profile));
                return Ok(())
            }

            if args.cmd_import {
                // reverse(?) transfer a note
                let mut from_args = args.clone();
                from_args.cmd_transfer = args.cmd_import;
                from_args.cmd_import = false;
                from_args.flag_profile = args.arg_name[0].clone();
                from_args.arg_name[0] = args.flag_profile.clone();
                
                let (mut from_profile, from_fingerprint) = try!(ThecaProfile::new(
                    &from_args.flag_profile,
                    &from_args.flag_profile_folder,
                    &from_args.flag_key,
                    from_args.cmd_new_profile,
                    from_args.flag_encrypted,
                    from_args.flag_yes
                ));

                try!(parse_cmds(&mut from_profile, &mut from_args, &from_fingerprint));
                return Ok(())
            }

            if args.cmd_list_profiles {
                let profile_pathbuf = try!(find_profile_folder(&args.flag_profile_folder));
                let profile_folder: &Path = &profile_pathbuf;
                try!(profiles_in_folder(profile_folder));
                return Ok(())
            }

            // list
            if args.arg_id.is_empty() {
                try!(profile.list_notes(
                    args.flag_limit,
                    args.flag_condensed,
                    args.flag_json,
                    args.flag_datesort,
                    args.flag_reverse,
                    args.flag_search_body
                ));
                return Ok(())
            }
        }
    }

    Ok(())
}
