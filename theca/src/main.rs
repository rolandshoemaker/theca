extern crate libc;
extern crate time;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
// use serialize::{Encodable, Decodable, Encoder, json};
use rustc_serialize::{Encodable, Decodable, Encoder, json};
use time::{now_utc, strftime};
use docopt::Docopt;
use std::os;
use std::io::fs::PathExtensions;
use std::io::process::{InheritFd};
use std::io::{File, Truncate, Write, Open, ReadWrite, TempDir, Command, SeekSet};

pub use self::libc::{
    STDIN_FILENO,
    STDOUT_FILENO,
    STDERR_FILENO
};

static VERSION:  &'static str = "0.0.1-dev";

// mod c {
//     extern crate libc;
//     pub use self::libc::{
//         c_int,
//         c_ushort,
//         c_ulong,
//         STDOUT_FILENO
//     };
//     use std::mem::zeroed;
//     pub struct winsize {
//         pub ws_row: c_ushort,
//         pub ws_col: c_ushort
//     }
//     #[cfg(any(target_os = "linux", target_os = "android"))]
//     static TIOCGWINSZ: c_ulong = 0x5413;
//     #[cfg(any(target_os = "macos", target_os = "ios"))]
//     static TIOCGWINSZ: c_ulong = 0x40087468;
//     extern {
//         pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
//     }
//     pub unsafe fn dimensions() -> winsize {
//         let mut window: winsize = zeroed();
//         ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window as *mut winsize);
//         window
//     }
// }

// fn termsize() -> Option<(uint, uint)> {
//     let ws = unsafe { c::dimensions() };

//     if ws.ws_col == 0 || ws.ws_row == 0 {
//         None
//     }
//     else {
//         Some((ws.ws_col as uint, ws.ws_row as uint))
//     }

// }

static USAGE: &'static str = "
theca - cli note taking tool

Usage:
    theca new-profile
    theca new-profile <name> [--encrypted]
    theca [options] [-c|-e] [-l LIMIT]
    theca [options] [-c|-e] <id>
    theca [options] [-c|-e] view <id>
    theca [options] add <title> [--started|--urgent] [-b BODY|--editor|-]
    theca [options] edit <id> [<title>] [--started|--urgent|--none] [-b BODY|--editor|-]
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
    --none                              No status.
    --started                           Started status.
    --urgent                            Urgent status.
    -b BODY                             Set body of the item to BODY.
    --editor                            Drop to $EDITOR to set/edit item body.
    -                                   Set body of the item to STDIN.
";

#[deriving(RustcDecodable, Show)]
struct Args {
    flag_config: Vec<String>,
    flag_profiles_folder: Vec<String>,
    flag_p: Vec<String>,
    cmd_new_profile: bool,
    cmd_view: bool,
    cmd_add: bool,
    cmd_edit: bool,
    cmd_del: bool,
    arg_name: String,
    flag_encrypted: bool,
    flag_c: bool,
    flag_e: bool,
    flag_l: Vec<uint>,
    arg_title: String,
    flag_started: bool,
    flag_urgent: bool,
    flag_none: bool,
    flag_b: Vec<String>,
    flag_editor: bool,
    cmd__: bool,
    arg_id: Vec<uint>,
    flag_h: bool,
    flag_v: bool
}

impl Args {
    fn check_rc(&mut self) {
    }
}

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

#[deriving(Copy)]
pub struct LineFormat {
    colsep: uint,
    id_width: uint,
    title_width: uint,
    status_width: uint,
    touched_width: uint
}

impl LineFormat {
    fn new(items: &Vec<ThecaItem>) -> LineFormat {
        // get terminal width, unused atm
        // let (width, height) = match termsize() {
        //     None => panic!(),
        //     Some((width, height)) => (width, height),
        // };

        // set minimums (header length) + colsep, this should probably do some other stuff?
        let mut line_format = LineFormat {colsep: 3, id_width:2, title_width:5, status_width:7, touched_width:7};

        for i in range(0, items.len()) {
            line_format.id_width = if items[i].id.to_string().len() > line_format.id_width {items[i].id.to_string().len()} else {line_format.id_width};
            if !items[i].body.is_empty() {
                line_format.title_width = if items[i].title.len()+4 > line_format.title_width {items[i].title.len()+4} else {line_format.title_width};
            } else {
                line_format.title_width = if items[i].title.len() > line_format.title_width {items[i].title.len()} else {line_format.title_width};
            }
            line_format.status_width = if items[i].status.len() > line_format.status_width {items[i].status.len()} else {line_format.status_width};
            line_format.touched_width = if items[i].last_touched.len() > line_format.touched_width {items[i].last_touched.len()} else {line_format.touched_width};
        }

        line_format
    }

    fn line_width(&self) -> uint {
        self.id_width+self.title_width+self.status_width+self.touched_width+(3*self.colsep)
    }
}

#[deriving(RustcDecodable, Clone)]
pub struct ThecaItem {
    id: uint,
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

impl ThecaItem {
    fn decrypt(&mut self, key: &str) {
    }

    fn print(& self, line_format: &LineFormat) {
        print!("{}", format_field(&self.id.to_string(), line_format.id_width));
        print!("{}", String::from_char(line_format.colsep, ' '));
        if !self.body.is_empty() {
            print!("(+) {}", format_field(&self.title, line_format.title_width-4));
        } else {
            print!("{}", format_field(&self.title, line_format.title_width));
        }
        print!("{}", String::from_char(line_format.colsep, ' '));
        print!("{}", format_field(&self.status, line_format.status_width));
        print!("{}", String::from_char(line_format.colsep, ' '));
        print!("{}", format_field(&self.last_touched, line_format.touched_width));
        print!("\n");
    }
}

#[deriving(RustcDecodable)]
pub struct ThecaProfile {
    current_id: uint,
    encrypted: bool,
    notes: Vec<ThecaItem>
}

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
    fn save_to_file(&mut self, args: &Args) {
        // set profile folder
        let mut profile_path = find_profile_folder(args);

        // set file name
        if !args.flag_p.is_empty() {
            profile_path.push(args.flag_p[0].to_string() + ".json");
        } else if args.cmd_new_profile {
            profile_path.push(args.arg_name.to_string() + ".json");
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
                    last_touched: strftime("%F %T", &now_utc()).ok().unwrap()
                });
            }
        }
        self.current_id += 1;
        println!("added");
    }

    fn delete_item(&mut self, id: uint) {
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

    fn edit_item(&mut self, id: uint, args: &Args) {
        let item_pos: uint = self.notes.iter()
            .position(|n| n.id == id)
            .unwrap();
        if !args.arg_title.is_empty() {
            // change title
            self.notes[item_pos].title = args.arg_title.to_string();
        } else if args.flag_started || args.flag_urgent || args.flag_none {
            // change status
            if args.flag_started {self.notes[item_pos].status = STARTED.to_string();} else if args.flag_urgent {self.notes[item_pos].status = URGENT.to_string();} else if args.flag_none {self.notes[item_pos].status = NOSTATUS.to_string();};
        } else if !args.flag_b.is_empty() || args.flag_editor || args.cmd__ {
            // change body
            if !args.flag_b.is_empty() {self.notes[item_pos].body = args.flag_b[0].to_string();} else if args.flag_editor {self.notes[item_pos].body = drop_to_editor(&self.notes[item_pos].body);} else if args.cmd__ {};
        }
        // update last_touched
        self.notes[item_pos].last_touched = strftime("%F %T", &now_utc()).ok().unwrap();
        println!("edited")
    }

    fn print_header(&mut self, line_format: &LineFormat) {
        print!("{}", format_field(&"id".to_string(), line_format.id_width));
        print!("{}", String::from_char(line_format.colsep, ' '));
        print!("{}", format_field(&"title".to_string(), line_format.title_width));
        print!("{}", String::from_char(line_format.colsep, ' '));
        print!("{}", format_field(&"status".to_string(), line_format.status_width));
        print!("{}", String::from_char(line_format.colsep, ' '));
        print!("{}", format_field(&"last touched".to_string(), line_format.touched_width));
        print!("\n");
        println!("{}", String::from_char(line_format.line_width(), '-'));
    }

    fn view_item(&mut self, id: uint, args: &Args, body: bool) {
        let item_pos: uint = self.notes.iter()
            .position(|n| n.id == id)
            .unwrap();
        let notes = vec![self.notes[item_pos].clone()];
        let line_format = LineFormat::new(&notes);
        if args.flag_e {
            self.print_header(&line_format);
        }
        notes[0].print(&line_format);
        if body && !notes[0].body.is_empty() {
            println!("{}", notes[0].body);
        }
    }

    fn list_items(&mut self, args: &Args) {
        let line_format = LineFormat::new(&self.notes);
        if args.flag_e {
            self.print_header(&line_format);
        }

        for i in range(0, self.notes.len()) {
            self.notes[i].print(&line_format);
        }
    }

    // should these searchs be for both title+body instead of seperate commands?
    // (probably...)

    // fn search_titles(&mut self, keyword: String) {
    // }

    // fn search_bodies(&mut self, keyword: String) {
    // }

    // fn search_titles_regex(&mut self, regex: String) {
    // }

    // fn search_bodies_regex(&mut self, regex: String) {
    // }
}

fn format_field(value: &String, width: uint) -> String {
    if value.len() > width && width > 3 {
        format!("{: <1$.1$}...", value, width-3)
    } else {
        format!("{: <1$.1$}", value, width)
    }
}

fn find_profile_folder(args: &Args) -> Path {
    if !args.flag_profiles_folder.is_empty() {
        Path::new(args.flag_p[0].to_string())
    } else {
        match os::homedir() {
            Some(ref p) => p.join(".theca"),
            None => Path::new(".").join(".theca")
        }
    }
}

fn drop_to_editor(contents: &String) -> String {
    // setup temporary directory
    let tmpdir = match TempDir::new("theca") {
        Ok(dir) => dir,
        Err(e) => panic!("couldn't create temporary directory: {}", e)
    };
    // setup temporary file to write/read
    let tmppath = tmpdir.path().join("something-unique");
    let mut tmpfile = match File::open_mode(&tmppath, Open, ReadWrite) {
        Ok(f) => f,
        Err(e) => panic!("File error: {}", e)
    };
    tmpfile.write_line(contents.as_slice()).ok().expect("Failed to write line to temp file");
    // we now have a temp file, at `tmppath`, that contains `contents`
    // first we need to know which one
    let editor = match os::getenv("VISUAL") {
        Some(val) => val,
        None => {
            match os::getenv("EDITOR") {
                Some(val) => val,
                None => panic!("Neither $VISUAL nor $EDITOR is set.")
            }
        }
    };
    // lets start `editor` and edit the file at `tmppath`
    // first we need to set STDIN, STDOUT, and STDERR to those that theca is
    // currently using so we can display the editor
    let mut editor_command = Command::new(editor);
    editor_command.arg(tmppath.display().to_string());
    editor_command.stdin(InheritFd(STDIN_FILENO));
    editor_command.stdout(InheritFd(STDOUT_FILENO));
    editor_command.stderr(InheritFd(STDERR_FILENO));
    let editor_proc = editor_command.spawn();
    match editor_proc.ok().expect("Couldn't launch editor").wait().is_ok() {
        true => {
            // finished editing, time to read `tmpfile` for the final output
            // seek to start of `tmpfile`
            tmpfile.seek(0, SeekSet).ok().expect("Can't seek to start of temp file");
            tmpfile.read_to_string().unwrap()
        }
        false => panic!("The editor broke")
    }
}

// this should be a method of ThecaProfile
fn build_profile(args: &Args) -> Result<ThecaProfile, String> {
    if args.cmd_new_profile {
        Ok(ThecaProfile {
            current_id: 0,
            encrypted: args.flag_encrypted,
            notes: vec![]
        })
    } else {
        // set profile folder
        let mut profile_path = find_profile_folder(args);

        // set profile name
        if !args.flag_p.is_empty() {
            profile_path.push(args.flag_p[0].to_string() + ".json");
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

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // Setup a ThecaProfile struct
    let mut profile = match build_profile(&args) {
        Ok(p) => p,
        Err(e) => panic!("{}", e)
    };

    // see what root command was used
    if args.cmd_add {
        // add a item
        let title = args.arg_title.to_string();
        let status = if args.flag_started {STARTED.to_string()} else if args.flag_urgent {URGENT.to_string()} else {NOSTATUS.to_string()};
        let body = if !args.flag_b.is_empty() {args.flag_b[0].to_string()} else if args.flag_editor {drop_to_editor(&"".to_string())} else {"".to_string()};
        profile.add_item(title, status, body);
    } else if args.cmd_edit {
        // edit a item
        let id = args.arg_id[0];
        profile.edit_item(id, &args);
    } else if args.cmd_del {
        // delete a item
        let id = args.arg_id[0];
        profile.delete_item(id);
    } else if args.flag_v {
        // display theca version
        println!("theca v{}", VERSION);
    } else if args.cmd_view {
        // view full item
        profile.view_item(args.arg_id[0], &args, true);
    } else if !args.cmd_view && !args.arg_id.is_empty() {
        // view short item
        profile.view_item(args.arg_id[0], &args, false);
    } else if !args.cmd_new_profile {
        // this should be the default for nothing
        profile.list_items(&args);
    }

    // save altered profile back to disk
    // this should only be triggered by commands that commit transactions to the profile
    if args.cmd_add || args.cmd_edit || args.cmd_del || args.cmd_new_profile {
        profile.save_to_file(&args);
    }
}
