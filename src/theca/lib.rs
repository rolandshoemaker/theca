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
#![allow(unstable)]

extern crate core;
extern crate libc;
extern crate time;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
extern crate regex;
extern crate crypto;
extern crate term;

// std lib imports
use std::os::{getenv};
use std::io::fs::{PathExtensions, mkdir};
use std::io::{File, Truncate, Write, Read, Open,
              stdin, USER_RWX};
use std::iter::{repeat};

// random things
use regex::{Regex};
use rustc_serialize::{Encodable, Encoder};
use rustc_serialize::json::{decode, as_pretty_json, PrettyEncoder};
use time::{now, strftime};

// crypto imports
use lineformat::{LineFormat};
use utils::c::{istty};
use utils::{drop_to_editor, pretty_line, format_field,
            get_yn_input, sorted_print, localize_last_touched_string,
            parse_last_touched, find_profile_folder, get_password};
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

static VERSION:  &'static str = "0.7.0-dev";

#[derive(RustcDecodable, Show, Clone)]
pub struct Args {
    pub cmd_add: bool,
    pub cmd_clear: bool,
    pub cmd_del: bool,
    pub cmd_edit: bool,
    pub cmd_info: bool,
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

// statics statuses
static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

// static date formats for strp/strf time
static DATEFMT: &'static str = "%F %T %z";
static DATEFMT_SHORT: &'static str = "%F %T";

// a note within the profile
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
    fn print(&self, line_format: &LineFormat, search_body: bool) -> Result<(), ThecaError> {
        let column_seperator: String = repeat(' ').take(line_format.colsep).collect();
        print!("{}", format_field(&self.id.to_string(), line_format.id_width, false));
        print!("{}", column_seperator);
        match !self.body.is_empty() && !search_body {
            true => {
                print!("{}", format_field(&self.title, line_format.title_width-4, true));
                print!("{}", format_field(&" (+)".to_string(), 4, false));
            },
            false => {
                print!("{}", format_field(&self.title, line_format.title_width, true));
            }
        }
        print!("{}", column_seperator);
        if line_format.status_width != 0 {
            print!("{}", format_field(&self.status, line_format.status_width, false));
            print!("{}", column_seperator);
        }
        print!("{}", format_field(&try!(localize_last_touched_string(&*self.last_touched)), line_format.touched_width, false));
        print!("\n");
        if search_body {
            for l in self.body.lines() {
                println!("\t{}", l);
            }
        }
        Ok(())
    }
}

// root object for theca profile files
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
            let profile_path = try!(find_profile_folder(profile_folder));
            // if the folder doesn't exist, make it yo!
            if !profile_path.exists() {
                if !yes {
                    println!(
                        "{} doesn't exist, would you like to create it?",
                        profile_path.display()
                    );
                    if !try!(get_yn_input()) {specific_fail!("ok bye ♥".to_string());}
                }
                try!(mkdir(&profile_path, USER_RWX));
            }
            Ok((ThecaProfile {
                encrypted: encrypted,
                notes: vec![]
            }, 0u64))
        } else {
            // set profile folder
            let mut profile_path = try!(find_profile_folder(profile_folder));

            // set profile name
            profile_path.push(profile_name.to_string() + ".json");
            
            // attempt to read profile
            match profile_path.is_file() {
                false => {
                    if profile_path.exists() {
                        specific_fail!(format!(
                            "{} is not a file.",
                            profile_path.display()
                        ));
                    } else {
                        specific_fail!(format!(
                            "{} does not exist.",
                            profile_path.display()
                        ));
                    }
                }
                true => {
                    let mut file = try!(File::open_mode(&profile_path, Open, Read));
                    let contents_buf = try!(file.read_to_end());
                    // decrypt the file if flag_encrypted
                    let contents = match encrypted {
                        false => try!(String::from_utf8(contents_buf)),
                        true => {
                            let key = password_to_key(&key[]);
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
                    Ok((decoded, try!(profile_path.stat()).modified))
                }
            }
        }
    }

    /// remove all notes from the profile
    pub fn clear(&mut self, yes: bool) -> Result<(), ThecaError> {
        if !yes {
            println!("are you sure you want to delete all the notes in this profile?");
            if !try!(get_yn_input()) {specific_fail!("ok bye ♥".to_string());}
        }
        self.notes.truncate(0);
        Ok(())
    }

    // FIXME
    /// save the profile back to file (either plaintext or encrypted)
    pub fn save_to_file(
        &mut self,
        args: &Args,
        fingerprint: &u64
    ) -> Result<(), ThecaError> {
        // set profile folder
        let mut profile_path = try!(find_profile_folder(&args.flag_profile_folder));

        // set file name
        match args.cmd_new_profile {
            true => profile_path.push(args.arg_name[0].to_string() + ".json"),
            false => profile_path.push(args.flag_profile.to_string() + ".json")
        };

        if args.cmd_new_profile && profile_path.exists() && !args.flag_yes {
            println!("profile {} already exists would you like to overwrite it?", profile_path.display());
            if !try!(get_yn_input()) {specific_fail!("ok bye ♥".to_string());}
        }

        if fingerprint > &0u64 {
            let new_fingerprint = try!(profile_path.stat()).modified;
            if &new_fingerprint != fingerprint && !args.flag_yes {
                println!("changes have been made to the profile '{}' on disk since it was loaded, would you like to attempt to merge them?", args.flag_profile);
                if !try!(get_yn_input()) {specific_fail!("ok bye ♥".to_string());}
                let mut new_args = args.clone();
                if args.flag_editor { 
                    new_args.flag_editor = false;
                    new_args.flag_body[0] = match self.notes.last() {
                        Some(n) => n.body.clone(),
                        None => "".to_string()
                    };
                }
                let (mut changed_profile, changed_fingerprint) = try!(ThecaProfile::new(
                    &new_args.flag_profile,
                    &new_args.flag_profile_folder,
                    &new_args.flag_key,
                    new_args.cmd_new_profile,
                    new_args.flag_encrypted,
                    new_args.flag_yes
                ));
                try!(parse_cmds(&mut changed_profile, &mut new_args, &changed_fingerprint));
                try!(changed_profile.save_to_file(&new_args, &0u64));
                return Ok(())
            }
        }

        // open file
        let mut file = try!(File::open_mode(&profile_path, Truncate, Write));

        // encode to buffer
        let mut buffer: Vec<u8> = Vec::new();
        {
            let mut encoder = PrettyEncoder::new(&mut buffer);
            try!(self.encode(&mut encoder));
        }

        // encrypt json if its an encrypted profile
        if self.encrypted {
            let key = password_to_key(&*args.flag_key);
            buffer = try!(encrypt(
                &*buffer,
                &*key
            ));
        }

        // write buffer to file
        try!(file.write(&*buffer));

        Ok(())
    }

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
        // let (mut trans_profile, trans_fingerprint) = try!(ThecaProfile::new(&mut trans_args));
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
                            let (started, urgent) = (n.status == STARTED, n.status == URGENT);
                            trans_profile.add_item(
                                &n.title,
                                &vec![n.body.clone()],
                                started,
                                urgent,
                                false,
                                false
                            )
                        }
                        ).is_some() {
            true =>  {
                match self.notes.iter().position(|n| n.id == args.arg_id[0])
                                   .map(|e| self.notes.remove(e)).is_some() {
                    true => try!(trans_profile.save_to_file(&trans_args, &trans_fingerprint)),
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
            "transfered {}: note {} -> {}: note {}",
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
    pub fn add_item(
        &mut self,
        title: &String,
        body: &Vec<String>,
        started: bool,
        urgent: bool,
        use_stdin: bool,
        use_editor: bool
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
            true => try!(stdin().lock().read_to_string())
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
        println!("note {} added", new_id+1);
        Ok(())
    }

    /// delete an item from the profile
    pub fn delete_item(&mut self, id: &Vec<usize>) {
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
    pub fn edit_item(
        &mut self,
        id: usize,
        title: &String,
        body: &Vec<String>,
        started: bool,
        urgent: bool,
        no_status: bool,
        use_stdin: bool,
        use_editor: bool
    ) -> Result<(), ThecaError> {
        // let id = args.arg_id[0];
        let item_pos: usize = match self.notes.iter().position(|n| n.id == id) {
            Some(i) => i,
            None => specific_fail!(format!("note {} doesn't exist", id))
        };
        if !title.is_empty() {
            match title.replace("\n", "") == "-" {
                true => match !use_stdin {
                    true => self.notes[item_pos].body = try!(stdin().lock().read_to_string()),
                    false => self.notes[item_pos].title = title.replace("\n", "").to_string()
                },
                false => self.notes[item_pos].title = title.replace("\n", "").to_string()
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
                true => try!(stdin().lock().read_to_string()),
                false => match use_editor {
                    true => {
                        match istty(STDOUT_FILENO) && istty(STDIN_FILENO) {
                            true => {
                                let new_body = try!(drop_to_editor(&self.notes[item_pos].body));
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
        let started_s = self.notes.iter().filter(|n| n.status == "Started").count();
        let urgent_s = self.notes.iter().filter(|n| n.status == "Urgent").count();
        let tty = istty(STDOUT_FILENO);
        let min = match self.notes.iter().min_by(|n| match parse_last_touched(&*n.last_touched) {
            Ok(o) => o,
            Err(_) => now() // FIXME
        }) {
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => specific_fail!("last_touched is not properly formated".to_string())
        };
        let max = match self.notes.iter().max_by(|n| match parse_last_touched(&*n.last_touched) {
            Ok(o) => o,
            Err(_) => now() // FIXME
        }) {
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => specific_fail!("last_touched is not properly formated".to_string())
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
        try!(pretty_line("note ages: ", &format!("oldest: {}, newest: {}\n", min, max), tty));
        Ok(())
    }

    /// print a full item
    pub fn view_item(
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
                            &format!("{}\n", try!(localize_last_touched_string(&*self.notes[note_pos].last_touched))),
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
                            &format!("{}\n\n", try!(localize_last_touched_string(&*self.notes[note_pos].last_touched))),
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
            true => println!("{}", as_pretty_json(&self.notes[note_pos].clone()))
        }
        Ok(())
    }

    /// print all items in the profile
    pub fn list_items(
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

    /// print items search for in the profile
    pub fn search_items(
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
                let re = match Regex::new(&pattern[]) {
                    Ok(r) => r,
                    Err(e) => specific_fail!(format!("regex error: {}.", e.msg))
                };
                self.notes.iter().filter(|n| match search_body {
                    true => re.is_match(&*n.body),
                    false => re.is_match(&*n.title)
                }).map(|n| n.clone()).collect()
            },
            false => {
                self.notes.iter().filter(|n| match search_body {
                    true => n.body.contains(&pattern[]),
                    false => n.title.contains(&pattern[])
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
    match getenv("THECA_DEFAULT_PROFILE") {
        Some(val) => {
            if args.flag_profile.is_empty() {
                args.flag_profile = val;
            }
        },
        None => ()
    };

    match getenv("THECA_PROFILE_FOLDER") {
        Some(val) => {
            if args.flag_profile_folder.is_empty() {
                args.flag_profile_folder = val;
            }
        },
        None => ()
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

pub fn parse_cmds(profile: &mut ThecaProfile, args: &mut Args, profile_fingerprint: &u64) -> Result<(), ThecaError> {
    // view
    if !args.arg_id.is_empty() && !args.cmd_del && !args.cmd_edit && !args.cmd_transfer {
        try!(profile.view_item(
            args.arg_id[0],
            args.flag_json,
            args.flag_condensed
        ));
        return Ok(())
    }

    // search
    if args.cmd_search {
        try!(profile.search_items(
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
    if args.cmd_info { try!(profile.stats(&args.flag_profile)); return Ok(()) }

    // misc
    if args.flag_version { println!("theca v{}", VERSION); return Ok(()) }

    // add
    if args.cmd_add {
        try!(profile.add_item(
            &args.arg_title,
            &args.flag_body,
            args.flag_started,
            args.flag_urgent,
            args.cmd__,
            args.flag_editor
        )); 
    }

    // edit    
    if args.cmd_edit {
        try!(profile.edit_item(
            args.arg_id[0],
            &args.arg_title,
            &args.flag_body,
            args.flag_started,
            args.flag_urgent,
            args.flag_none,
            args.cmd__,
            args.flag_editor
        ));
    }
    
    // delete    
    if args.cmd_del { profile.delete_item(&args.arg_id); }

    // transfer
    if args.cmd_transfer { try!(profile.transfer_note(args)); }

    // clear
    if args.cmd_clear { try!(profile.clear(args.flag_yes)); }

    // list
    if args.arg_id.is_empty() && !args.cmd_add && !args.cmd_edit && !args.cmd_del &&
       !args.cmd_transfer && !args.cmd_clear && !args.cmd_new_profile {
        try!(profile.list_items(
            args.flag_limit,
            args.flag_condensed,
            args.flag_json,
            args.flag_datesort,
            args.flag_reverse,
            args.flag_search_body
        ));
        return Ok(())
    }

    if args.cmd_new_profile {
        if args.cmd_new_profile && args.arg_name.is_empty() {
            args.arg_name.push("default".to_string())
        }
        println!("creating profile '{}'", args.arg_name[0]);
    }

    // new-profile, add, edit, del, transfer, clear
    try!(profile.save_to_file(args, profile_fingerprint));

    Ok(())
}
