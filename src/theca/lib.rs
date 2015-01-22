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
use std::os::{getenv, homedir};
use std::io::fs::{PathExtensions, mkdir};
use std::io::{File, Truncate, Write, Read, Open,
              stdin, USER_RWX};
use std::iter::{repeat};
use std::cmp::{Ordering};

// random things
use regex::{Regex};
use rustc_serialize::{Encodable, Encoder, json};
use time::{now, strftime, strptime, at};
use term::attr::Attr::{Bold};

// crypto imports
use utils::{termsize, drop_to_editor, get_password, pretty_line, format_field,
            get_yn_input};
use errors::{ThecaError, GenericError};
use crypt::{encrypt, decrypt, password_to_key};

pub use self::libc::{
    STDIN_FILENO,
    STDOUT_FILENO,
    STDERR_FILENO
};

#[macro_use] pub mod errors;
pub mod utils;
pub mod crypt;

#[derive(RustcDecodable, Show, Clone)]
pub struct Args {
    pub cmd_new_profile: bool,
    pub cmd_search: bool,
    pub cmd_add: bool,
    pub cmd_edit: bool,
    pub cmd_del: bool,
    pub cmd_clear: bool,
    pub cmd_info: bool,
    pub cmd_transfer: bool,
    pub arg_id: Vec<usize>,
    pub flag_v: bool,
    cmd__: bool,
    arg_name: String,
    arg_pattern: String,
    arg_title: String,
    flag_profile_folder: String,
    flag_p: String,
    flag_regex: bool,
    flag_body: bool,
    flag_reverse: bool,
    flag_encrypted: bool,
    flag_key: String,
    flag_c: bool,
    flag_l: usize,
    flag_datesort: bool,
    flag_started: bool,
    flag_urgent: bool,
    flag_none: bool,
    flag_b: String,
    flag_editor: bool,
    flag_h: bool,
    flag_y: bool,
    flag_append: String,
    flag_prepend: String
}

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

static DATEFMT: &'static str = "%F %T %z";
static DATEFMT_SHORT: &'static str = "%F %T";

#[derive(Copy)]
pub struct LineFormat {
    colsep: usize,
    id_width: usize,
    title_width: usize,
    status_width: usize,
    touched_width: usize
}

impl LineFormat {
    fn new(items: &Vec<ThecaItem>, args: &Args) -> Result<LineFormat, ThecaError> {
        // get termsize :>
        let console_width = termsize();

        // set colsep
        let colsep = match args.flag_c {
            true => 1,
            false => 2
        };

        let mut line_format = LineFormat {colsep: colsep, id_width:0, title_width:0,
                                          status_width:0, touched_width:0};

        // get length of longest id string
        line_format.id_width = match items.iter().max_by(|n| n.id.to_string().len()) {
            Some(w) => w.id.to_string().len(),
            None => 0
        };
        // if longest id is 1 char and we are using extended printing
        // then set id_width to 2 so "id" isn't truncated
        if line_format.id_width < 2 && !args.flag_c {line_format.id_width = 2;}

        // get length of longest title string
        line_format.title_width = match items.iter().max_by(|n| n.title.len()) {
            Some(w) => {
                if items.iter().any(|n| n.body.len() > 0) {
                    w.title.len()+4
                } else {
                    w.title.len()
                }
            },
            None => 0
        };
        // if using extended and longest title is less than 5 chars
        // set title_width to 5 so "title" won't be truncated
        if line_format.title_width < 5 && !args.flag_c {line_format.title_width = 5;}

        // status length stuff
        line_format.status_width = match items.iter().any(|n| n.status.len() > 0) {
            true => {
                match args.flag_c {
                    // expanded print, get longest status (7 or 6 / started or urgent)
                    false => {
                        match items.iter().max_by(|n| n.status.len()) {
                            Some(w) => w.status.len(),
                            None => {
                                0
                            }
                        }
                    },
                    // only display first char of status (e.g. S or U) for condensed print
                    true => 1
                }
            },
            // no items have statuses so truncate column
            false => {
                0
            }
        };

        // last_touched has fixed string length so no need for silly iter stuff
        line_format.touched_width = match args.flag_c {
            true => 10, // condensed
            false => 19 // expanded
        };

        // check to make sure our new line format isn't bigger than the console
        let line_width = line_format.line_width();
        if console_width > 0 && line_width > console_width &&
           (line_format.title_width-(line_width-console_width)) > 0 {
            // if it is trim text from the title width since it is always the biggest...
            // if there isn't any statuses, also give the title the colsep char space
            line_format.title_width -= match line_format.status_width == 0 {
                true => (line_width - console_width) + 2,
                false => line_width - console_width
            };
        }

        Ok(line_format)
    }

    fn line_width(&self) -> usize {
        let columns = match self.status_width == 0 {
            true => 2*self.colsep,
            false => 3*self.colsep
        };
        self.id_width+self.title_width+self.status_width+self.touched_width+columns
    }
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct ThecaItem {
    id: usize,
    title: String,
    status: String,
    body: String,
    last_touched: String
}

impl ThecaItem {
    fn print(&self, line_format: &LineFormat, body_search: bool) -> Result<(), ThecaError> {
        let column_seperator: String = repeat(' ').take(line_format.colsep).collect();
        print!("{}", format_field(&self.id.to_string(), line_format.id_width, false));
        print!("{}", column_seperator);
        let mut title_str = self.title.to_string();
        if !self.body.is_empty() && !body_search {
            title_str = "(+) ".to_string()+&*title_str;
        }
        print!("{}", format_field(&title_str, line_format.title_width, true));
        print!("{}", column_seperator);
        if line_format.status_width != 0 {
            print!("{}", format_field(&self.status, line_format.status_width, false));
            print!("{}", column_seperator);
        }
        print!("{}", format_field(&try!(localize_last_touched_string(&*self.last_touched)), line_format.touched_width, false));
        print!("\n");
        if body_search {
            for l in self.body.lines() {
                println!("\t{}", l);
            }
        }
        Ok(())
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct ThecaProfile {
    encrypted: bool,
    pub notes: Vec<ThecaItem>
}

impl ThecaProfile {
    pub fn new(args: &Args) -> Result<ThecaProfile, ThecaError> {
        if args.cmd_new_profile {
            let profile_path = try!(find_profile_folder(args));
            // if the folder doesn't exist, make it yo!
            if !profile_path.exists() {
                if !args.flag_y {
                    println!(
                        "{} doesn't exist, would you like to create it?",
                        profile_path.display()
                    );
                    if !try!(get_yn_input()) {specific_fail!("ok bye".to_string());}
                }
                try!(mkdir(&profile_path, USER_RWX));
            }
            Ok(ThecaProfile {
                encrypted: args.flag_encrypted,
                notes: vec![]
            })
        } else {
            // set profile folder
            let mut profile_path = try!(find_profile_folder(args));

            // set profile name
            if !args.flag_p.is_empty() {
                profile_path.push(args.flag_p.to_string() + ".json");
            } else {
                profile_path.push("default".to_string() + ".json");
            }

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
                    if args.cmd_info {
                        println!("# Loading {}", profile_path.display());
                    }
                    let mut file = try!(File::open_mode(&profile_path, Open, Read));
                    let contents_buf = try!(file.read_to_end());
                    // decrypt the file if flag_encrypted
                    let contents = match args.flag_encrypted {
                        false => try!(String::from_utf8(contents_buf)),
                        true => {
                            let (key, iv) = password_to_key(&*args.flag_key);
                                try!(String::from_utf8(try!(decrypt(
                                    &*contents_buf,
                                    &*key,
                                    &*iv
                                ))))
                        }
                    };
                    let decoded: ThecaProfile = match json::decode(&*contents) {
                        Ok(s) => s,
                        Err(_) => specific_fail!(format!(
                            "Invalid JSON in {}",
                            profile_path.display()
                        ))
                    };
                    Ok(decoded)
                }
            }
        }
    }

    pub fn clear(&mut self, args: &Args) -> Result<(), ThecaError> {
        println!("are you sure you want to delete all the notes in this profile?");
        if !args.flag_y && !try!(get_yn_input()) {specific_fail!("ok, bye".to_string());}
        self.notes.truncate(0);
        Ok(())
    }

    pub fn save_to_file(&mut self, args: &Args) -> Result<(), ThecaError> {
        // set profile folder
        let mut profile_path = try!(find_profile_folder(args));

        // set file name
        // this needs some work.
        if !args.flag_p.is_empty() {
            profile_path.push(args.flag_p.to_string() + ".json");
        } else if args.cmd_new_profile && !args.arg_name.is_empty() {
            profile_path.push(args.arg_name.to_string() + ".json");
        } else {
            profile_path.push("default".to_string() + ".json");
        }

        // open file
        let mut file = try!(File::open_mode(&profile_path, Truncate, Write));

        // encode to buffer
        let mut buffer: Vec<u8> = Vec::new();
        {
            let mut encoder = json::PrettyEncoder::new(&mut buffer);
            try!(self.encode(&mut encoder));
        }

        // encrypt json if its an encrypted profile
        if self.encrypted {
            let (key, iv) = password_to_key(&*args.flag_key);
            buffer = try!(encrypt(
                &*buffer,
                &*key,
                &*iv
            ));
        }

        // write buffer to file
        try!(file.write(&*buffer));

        Ok(())
    }

    pub fn import_note(&mut self, note: ThecaItem) -> Result<(), ThecaError> {
        let new_id = match self.notes.last() {
            Some(n) => n.id,
            None => 0
        };
        self.notes.push(ThecaItem {
            id: new_id + 1,
            title: note.title.clone(),
            status: note.status.clone(),
            body: note.body.clone(),
            last_touched: try!(strftime(DATEFMT, &now()))
        });
        Ok(())
    }

    pub fn transfer_note(&mut self, args: &Args) -> Result<(), ThecaError> {
        if args.flag_p == args.arg_name {
            specific_fail!(format!(
                "cannot transfer a note from a profile to itself ({} -> {})",
                args.flag_p,
                args.arg_name
            ));
        }

        let mut trans_args = args.clone();
        trans_args.flag_p = args.arg_name.clone();
        let mut trans_profile = try!(ThecaProfile::new(&trans_args));

        match self.notes.iter().find(|n| n.id == args.arg_id[0])
                        .map(|n| trans_profile.import_note(n.clone())).is_some() {
            true =>  {
                match self.notes.iter().position(|n| n.id == args.arg_id[0])
                                   .map(|e| self.notes.remove(e)).is_some() {
                    true => try!(trans_profile.save_to_file(&trans_args)),
                    false => specific_fail!(format!(
                        "couldn't remove note {} in {}, aborting nothing will be saved",
                        args.arg_id[0],
                        args.flag_p
                    ))
                };
            },
            false => specific_fail!(format!(
                "could not transfer note {} from {} -> {}",
                args.arg_id[0],
                args.flag_p,
                args.arg_name
            ))
        };
        Ok(())
    }

    pub fn add_item(&mut self, args: &Args) -> Result<(), ThecaError> {
        let title = args.arg_title.replace("\n", "").to_string();
        let status = if args.flag_started {
            STARTED.to_string()
        } else if args.flag_urgent {
            URGENT.to_string()
        } else {
            NOSTATUS.to_string()
        };
        let body = if !args.flag_b.is_empty() {
            args.flag_b.to_string()
        } else if args.flag_editor {
            try!(drop_to_editor(&"".to_string()))
        } else if args.cmd__ {
            try!(stdin().lock().read_to_string())
        } else {
            "".to_string()
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
        println!("note added");
        Ok(())
    }

    pub fn delete_item(&mut self, id: &usize) {
        let remove = self.notes.iter()
            .position(|n| n.id == *id)
            .map(|e| self.notes.remove(e))
            .is_some();
        match remove {
            true => {
                println!("note {} removed", id);
            }
            false => {
                println!("note {} doesn't exist", id);
            }
        }
    }

    pub fn edit_item(&mut self, args: &Args) -> Result<(), ThecaError> {
        let id = args.arg_id[0];
        let item_pos: usize = match self.notes.iter()
                                              .position(|n| n.id == id) {
                Some(i) => i,
                None => specific_fail!(format!("note {} doesn't exist", id))
            };
        if !args.arg_title.is_empty() {
            // change title
            self.notes[item_pos].title = args.arg_title.replace("\n", "").to_string();
        } else if !args.flag_prepend.is_empty() || !args.flag_append.is_empty() {
            self.notes[item_pos].title = format!(
                "{}{}{}",
                args.flag_prepend,
                self.notes[item_pos].title,
                args.flag_append
            );
        }
        if args.flag_started || args.flag_urgent || args.flag_none {
            // change status
            if args.flag_started {
                self.notes[item_pos].status = STARTED.to_string();
            } else if args.flag_urgent {
                self.notes[item_pos].status = URGENT.to_string();
            } else if args.flag_none {
                self.notes[item_pos].status = NOSTATUS.to_string();
            }
        }
        if !args.flag_b.is_empty() || args.flag_editor || args.cmd__ {
            // change body
            if !args.flag_b.is_empty() {
                self.notes[item_pos].body = args.flag_b.to_string();
            } else if args.flag_editor {
                if args.flag_encrypted && !args.flag_y {
                    // leak to disk warning
                    println!(
                        "{}\n{}\n{}\n{}\n{}",
                        "*******************************************************************",
                        "* Warning: this will write the decrypted note to disk temporarily *",
                        "* for editing, it will be deleted when you are done, but this     *",
                        "* increases the chance that it may be recovered at a later date.  *",
                        "*******************************************************************");
                    println!("Do you want to continue?");
                    if !try!(get_yn_input()) {specific_fail!("ok, bye".to_string());}
                }
                self.notes[item_pos].body = try!(drop_to_editor(&self.notes[item_pos].body));
            } else if args.cmd__ {
                try!(stdin().lock().read_to_string());
            }
        }
        // update last_touched
        self.notes[item_pos].last_touched = try!(strftime(DATEFMT, &now()));
        println!("edited");
        Ok(())
    }

    pub fn stats(&mut self, args: &Args) -> Result<(), ThecaError> {
        let no_s = self.notes.iter().filter(|n| n.status == "").count();
        let started_s = self.notes.iter().filter(|n| n.status == "Started").count();
        let urgent_s = self.notes.iter().filter(|n| n.status == "Urgent").count();
        let color = termsize() > 0;
        let min = match self.notes.iter().min_by(|n| at(strptime(&*n.last_touched, DATEFMT).unwrap().to_timespec())) { // FIXME
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => specific_fail!("bad".to_string())
        };
        let max = match self.notes.iter().max_by(|n| at(strptime(&*n.last_touched, DATEFMT).unwrap().to_timespec())) { // FIXME
            Some(n) => try!(localize_last_touched_string(&*n.last_touched)),
            None => specific_fail!("bad".to_string())
        };
        try!(pretty_line("name: ", &format!("{}\n", args.flag_p), color));
        try!(pretty_line("encrypted: ", &format!("{}\n", self.encrypted), color));
        try!(pretty_line("notes: ", &format!("{}\n", self.notes.len()), color));
        try!(pretty_line("statuses: ", &format!(
            "none: {}, started: {}, urgent: {}\n",
            no_s,
            started_s,
            urgent_s
        ), color));
        try!(pretty_line("note ages: ", &format!("oldest: {}, newest: {}\n", min, max), color));
        Ok(())
    }

    pub fn view_item(&mut self, args: &Args) -> Result<(), ThecaError> {
        let id = args.arg_id[0];
        let note_pos = match self.notes.iter().position(|n| n.id == id) {
            Some(i) => i,
            None => specific_fail!(format!("note {} doesn't exist", id))
        };
        let color = termsize() > 0;

        match args.flag_c {
            true => {
                try!(pretty_line("id: ", &format!(
                    "{}\n",
                    self.notes[note_pos].id),
                    color
                ));
                try!(pretty_line("title: ", &format!(
                    "{}\n",
                    self.notes[note_pos].title),
                    color
                ));
                if !self.notes[note_pos].status.is_empty() {
                    try!(pretty_line("status: ", &format!(
                        "{}\n",
                        self.notes[note_pos].status),
                        color
                    ));
                }
                try!(pretty_line(
                    "last touched: ",
                    &format!("{}\n", try!(localize_last_touched_string(&*self.notes[note_pos].last_touched))),
                    color
                ));
            },
            false => {
                try!(pretty_line("id\n--\n", &format!(
                    "{}\n\n",
                    self.notes[note_pos].id),
                    color
                ));
                try!(pretty_line("title\n-----\n", &format!(
                    "{}\n\n",
                    self.notes[note_pos].title),
                    color
                ));
                if !self.notes[note_pos].status.is_empty() {
                    try!(pretty_line(
                        "status\n------\n",
                        &format!("{}\n\n", self.notes[note_pos].status),
                        color
                    ));
                }
                try!(pretty_line(
                    "last touched\n------------\n",
                    &format!("{}\n\n", try!(localize_last_touched_string(&*self.notes[note_pos].last_touched))),
                    color
                ));
            }
        };

        // body
        if !self.notes[note_pos].body.is_empty() {
            match args.flag_c {
                true => {
                    try!(pretty_line("body: ", &format!(
                        "{}\n",
                        self.notes[note_pos].body),
                        color
                    ));
                },
                false => {
                    try!(pretty_line("body\n----\n", &format!(
                        "{}\n\n",
                        self.notes[note_pos].body),
                        color
                    ));
                }
            };
        }
        Ok(())
    }

    pub fn list_items(&mut self, args: &Args) -> Result<(), ThecaError> {
        if self.notes.len() > 0 {
            try!(sorted_print(&mut self.notes.clone(), args));
        } else {
            println!("this profile is empty");
        }
        Ok(())
    }

    pub fn search_items(&mut self, args: &Args) -> Result<(), ThecaError> {
        let pattern = &*args.arg_pattern;
        let notes: Vec<ThecaItem> = match args.flag_regex {
            true => {
                let re = match Regex::new(pattern) {
                    Ok(r) => r,
                    Err(e) => specific_fail!(format!("regex error: {}.", e.msg))
                };
                self.notes.iter().filter(|n| match args.flag_body {
                    true => re.is_match(&*n.body),
                    false => re.is_match(&*n.title)
                }).map(|n| n.clone()).collect()
            },
            false => {
                self.notes.iter().filter(|n| match args.flag_body {
                    true => n.body.contains(pattern),
                    false => n.title.contains(pattern)
                }).map(|n| n.clone()).collect()
            }
        };
        if notes.len() > 0 {
            try!(sorted_print(&mut notes.clone(), args));
        } else {
            println!("nothing found");
        }
        Ok(())
    }
}

fn print_header(line_format: &LineFormat) -> Result<(), ThecaError> {
    let mut t = match term::stdout() {
        Some(t) => t,
        None => specific_fail!("could not retrieve standard output.".to_string())
    };
    let column_seperator: String = repeat(' ').take(line_format.colsep).collect();
    let header_seperator: String = repeat('-').take(line_format.line_width()).collect();
    let color = termsize() > 0;
    let status = match line_format.status_width == 0 {
        true => "".to_string(),
        false => format_field(
            &"status".to_string(),
            line_format.status_width,
            false
        )+&*column_seperator
    };
    if color {try!(t.attr(Bold));}
    try!(write!(
                t, 
                "{1}{0}{2}{0}{3}{4}\n{5}\n",
                column_seperator,
                format_field(&"id".to_string(), line_format.id_width, false),
                format_field(&"title".to_string(), line_format.title_width, false),
                status,
                format_field(&"last touched".to_string(), line_format.touched_width, false),
                header_seperator
            ));
    if color {try!(t.reset());}
    Ok(())
}

fn sorted_print(notes: &mut Vec<ThecaItem>, args: &Args) -> Result<(), ThecaError> {
    let line_format = try!(LineFormat::new(notes, args));
    let limit = match args.flag_l != 0 && notes.len() >= args.flag_l {
        true => args.flag_l,
        false => notes.len()
    };
    if !args.flag_c {
        try!(print_header(&line_format));
    }
    // im not really sure why this... works?
    if args.flag_datesort {
        notes.sort_by(|a, b| match cmp_last_touched(&*a.last_touched, &*b.last_touched) {
            Ok(o) => o,
            Err(_) => a.last_touched.cmp(&b.last_touched) // FIXME(?)
            // Err(_) => Ordering::Equal                  // FIXME(?)
        });
    }
    match args.flag_reverse {
        false => for n in notes[0..limit].iter() {
            try!(n.print(&line_format, args.flag_body));
        },
        true => for n in notes[notes.len()-limit..notes.len()].iter().rev() {
            try!(n.print(&line_format, args.flag_body));
        }
    };
    Ok(())
}

fn find_profile_folder(args: &Args) -> Result<Path, ThecaError> {
    if !args.flag_profile_folder.is_empty() {
        Ok(Path::new(args.flag_profile_folder.to_string()))
    } else {
        match homedir() {
            Some(ref p) => Ok(p.join(".theca")),
            None => specific_fail!("failed to find your home directory".to_string())
        }
    }
}

pub fn setup_args(args: &mut Args) -> Result<(), ThecaError> {
    match getenv("THECA_DEFAULT_PROFILE") {
        Some(val) => {
            if args.flag_p.is_empty() {
                args.flag_p = val;
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
    if args.flag_encrypted && args.flag_key.is_empty() && !args.flag_y {
        args.flag_key = try!(get_password());
    }

    if args.flag_p.is_empty() {
        args.flag_p = "default".to_string();
    }

    Ok(())
}

fn parse_last_touched(lt: &str) -> Result<time::Tm, ThecaError> {
    Ok(at(try!(strptime(lt, DATEFMT)).to_timespec()))
}

fn localize_last_touched_string(lt: &str) -> Result<String, ThecaError> {
    let t = try!(parse_last_touched(lt));
    Ok(try!(strftime(DATEFMT_SHORT, &t)))
}

fn cmp_last_touched(a: &str, b: &str) -> Result<Ordering, ThecaError> {
    let a_tm = try!(parse_last_touched(a));
    let b_tm = try!(parse_last_touched(b));
    Ok(a_tm.cmp(&b_tm))
}
