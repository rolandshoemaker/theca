#![allow(unstable)]

extern crate libc;
extern crate time;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
extern crate regex;
extern crate crypto;
extern crate term;

// std lib imports...
use std::os::{getenv, homedir};
use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::process::{InheritFd};
use std::io::{File, Truncate, Write, Read, Open, ReadWrite,
              TempDir, Command, SeekSet, stdin, USER_RWX};
use std::iter::{repeat};

// random things
use regex::{Regex};
use rustc_serialize::{Encodable, Decodable, Encoder, json};
use time::{now_utc, strftime, get_time};
use docopt::Docopt;
use term::attr::Attr::{Bold};

// crypto imports
use crypt::{encrypt, decrypt, password_to_key};

pub use self::libc::{
    STDIN_FILENO,
    STDOUT_FILENO,
    STDERR_FILENO
};


pub mod crypt;

static VERSION:  &'static str = "0.4.5-dev";

mod c {
    extern crate libc;
    pub use self::libc::{
        c_int,
        c_ushort,
        c_ulong,
        STDOUT_FILENO
    };
    use std::mem::zeroed;
    pub struct Winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    static TIOCGWINSZ: c_ulong = 0x5413;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    static TIOCGWINSZ: c_ulong = 0x40087468;
    extern {
        pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }
    pub unsafe fn dimensions() -> Winsize {
        let mut window: Winsize = zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window as *mut Winsize);
        window
    }
}

fn termsize() -> Option<usize> {
    let ws = unsafe { c::dimensions() };
    if ws.ws_col == 0 || ws.ws_row == 0 {
        None
    }
    else {
        Some(ws.ws_col as usize)
    }
}

static USAGE: &'static str = "
theca - cli note taking tool

Usage:
    theca [options] info
    theca [options] new-profile <name>
    theca [options] [-c] [-l LIMIT] [--reverse]
    theca [options] [-c] <id>
    theca [options] [-c] search [--body] [-l LIMIT] [--reverse] <pattern>
    theca [options] add <title> [--started|--urgent] [-b BODY|--editor|-]
    theca [options] edit <id> [<title>] [--started|--urgent|--none] [-b BODY|--editor|-]
    theca [options] del <id>
    theca (-h | --help)
    theca --version

Options:
    -h, --help                          Show this screen.
    -v, --version                       Show the version of theca.
    --profiles-folder PROFILEPATH       Path to folder container profile.json files.
    -p PROFILE, --profile PROFILE       Specify non-default profile.
    -c, --condensed                     Use the condensed print format.
    --encrypted                         Specifies using an encrypted profile.
    -k KEY, --key KEY                   Encryption key to use for encryption/decryption,
                                        a prompt will be displayed if no key is provided.
    -l LIMIT                            Limit listing to LIMIT items [default: 0].
    --none                              No status. (default)
    --started                           Started status.
    --urgent                            Urgent status.
    -b BODY                             Set body of the item from BODY.
    --editor                            Drop to $EDITOR to set/edit item body.
    -                                   Set body of the item from STDIN.
";

#[derive(RustcDecodable, Show)]
struct Args {
    flag_profiles_folder: String,
    flag_p: String,
    cmd_new_profile: bool,
    cmd_search: bool,
    flag_body: bool,
    flag_reverse: bool,
    cmd_add: bool,
    cmd_edit: bool,
    cmd_del: bool,
    arg_name: String,
    arg_pattern: String,
    flag_encrypted: bool,
    flag_key: String,
    flag_c: bool,
    flag_l: usize,
    arg_title: String,
    flag_started: bool,
    flag_urgent: bool,
    flag_none: bool,
    flag_b: String,
    flag_editor: bool,
    cmd__: bool,
    arg_id: Vec<usize>,
    flag_h: bool,
    flag_v: bool,
    cmd_info: bool
}

impl Args {
    fn check_env(&mut self) {
        match getenv("THECA_DEFAULT_PROFILE") {
            Some(val) => {
                if self.flag_p.is_empty() {
                    self.flag_p = val;
                }
            },
            None => ()
        };
        match getenv("THECA_PROFILE_FOLDER") {
            Some(val) => {
                if self.flag_profiles_folder.is_empty() {
                    self.flag_profiles_folder = val;
                }
            },
            None => ()
        };
    }
}

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

#[derive(Copy)]
pub struct LineFormat {
    colsep: usize,
    id_width: usize,
    title_width: usize,
    status_width: usize,
    touched_width: usize
}

impl LineFormat {
    fn new(items: &Vec<ThecaItem>, args: &Args) -> LineFormat {
        // get termsize :>
        let console_width = match termsize() {
            None => panic!("Cannot retrieve terminal information"),
            Some(width) => width,
        };

        // set colsep
        let colsep = match args.flag_c {
            true => 1,
            false => 2
        };
        let mut line_format = LineFormat {colsep: colsep, id_width:0, title_width:0,
                                          status_width:0, touched_width:0};

        // get length of longest id string
        line_format.id_width = items.iter().max_by(|n| n.id.to_string().len())
                                    .unwrap().id.to_string().len();
        // if longest id is 1 char and we are using extended printing
        // then set id_width to 2 so "id" isn't truncated
        if line_format.id_width < 2 && !args.flag_c {line_format.id_width = 2;}

        // get length of longest title string
        line_format.title_width = items.iter().max_by(|n| n.title.len()).unwrap().title.len();
        // if any item has a body assume the longest one does too so add 4
        // to allow for use of "(+) " to indicate note body
        if items.iter().any(|n| n.body.len() > 0) {line_format.title_width += 4;}
        // if using extended and longest title is less than 5 chars
        // set title_width to 5 so "title" won't be truncated
        if line_format.title_width < 5 && !args.flag_c {line_format.title_width = 5;}

        // sstatus length stuff
        line_format.status_width = match items.iter().any(|n| n.status.len() > 0) {
            true => {
                match args.flag_c {
                    // expanded print, get longest status (7 or 6 / started or urgent)
                    true => items.iter().max_by(|n| n.status.len()).unwrap().status.len(),
                    // only display first char of status (e.g. S or U) for condensed print
                    false => 1
                }
            },
            // no items have statuses so truncate column
            false => 0
        };

        // last_touched has fixed string length so no need for silly iter stuff
        line_format.touched_width = match args.flag_c {
            true => 10, // condensed
            false => 19 // expanded
        };

        // check to make sure our new line format isn't bigger than the console
        let line_width = line_format.line_width();
        if line_width > console_width && (line_format.title_width-(line_width-console_width)) > 0 {
            // if it is trim text from the title width since it is always the biggest...
            line_format.title_width -= line_width - console_width;
        }

        line_format
    }

    fn line_width(&self) -> usize {
        self.id_width+self.title_width+self.status_width+self.touched_width+(3*self.colsep)
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
    fn print(&self, line_format: &LineFormat, args: &Args) {
        let column_seperator: String = repeat(' ').take(line_format.colsep).collect();
        print!("{}", format_field(&self.id.to_string(), line_format.id_width, false));
        print!("{}", column_seperator);
        let mut title_str = self.title.to_string();
        if !self.body.is_empty() && !args.flag_body {
            title_str = "(+) ".to_string()+title_str.as_slice();
        }
        print!("{}", format_field(&title_str, line_format.title_width, true));
        print!("{}", column_seperator);
        if args.flag_c && self.status.len() > 0 {
            print!("{}", format_field(
                &self.status.chars().nth(0).unwrap().to_string(),
                line_format.status_width,
                false)
            );
        } else {
            print!("{}", format_field(&self.status, line_format.status_width, false));
        }
        print!("{}", column_seperator);
        print!("{}", format_field(&self.last_touched, line_format.touched_width, false));
        print!("\n");
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct ThecaProfile {
    encrypted: bool,
    notes: Vec<ThecaItem>
}

impl ThecaProfile {
    fn new(args: &Args) -> Result<ThecaProfile, String> {
        if args.cmd_new_profile {
            let profile_path = find_profile_folder(args);
            // if the folder doesn't exist, make it yo!
            if !profile_path.exists() {
                match fs::mkdir(&profile_path, USER_RWX) {
                    Ok(_) => (),
                    Err(e) => panic!(
                        "Creating folder {} failed. {}",
                        profile_path.display(),
                        e.desc
                    )
                };
            }
            Ok(ThecaProfile {
                encrypted: args.flag_encrypted,
                notes: vec![]
            })
        } else {
            // set profile folder
            let mut profile_path = find_profile_folder(args);

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
                        Err(format!("{} is not a file.", profile_path.display()))
                    } else {
                        Err(format!("{} does not exist.", profile_path.display()))
                    }
                }
                true => {
                    if args.cmd_info {
                        println!("# Loading {}", profile_path.display());
                    }
                    let mut file = match File::open_mode(&profile_path, Open, Read) {
                        Ok(t) => t,
                        Err(e) => panic!("{}", e.desc)
                    };
                    let contents_buf = match file.read_to_end() {
                        Ok(t) => t,
                        Err(e) => panic!("{}", e.desc)
                    };
                    // decrypt the file if flag_encrypted
                    let contents = match args.flag_encrypted {
                        false => String::from_utf8(contents_buf).unwrap(),
                        true => {
                            let (key, iv) = password_to_key(args.flag_key.as_slice());
                                String::from_utf8(decrypt(
                                    contents_buf.as_slice(),
                                    key.as_slice(),
                                    iv.as_slice()
                                ).ok().unwrap()).unwrap()
                        }
                    };
                    let decoded: ThecaProfile = match json::decode(contents.as_slice()) {
                        Ok(s) => s,
                        Err(_) => panic!("Invalid JSON in {}", profile_path.display())
                    };
                    Ok(decoded)
                }
            }
        }
    }

    fn save_to_file(&mut self, args: &Args) {
        // set profile folder
        let mut profile_path = find_profile_folder(args);

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
        let mut file = match File::open_mode(&profile_path, Truncate, Write) {
            Ok(f) => f,
            Err(e) => panic!("File error: {}", e)
        };

        // encode to buffer
        let mut buffer: Vec<u8> = Vec::new();
        {
            let mut encoder = json::PrettyEncoder::new(&mut buffer);
            self.encode(&mut encoder).ok().expect("JSON encoding error.");
        }

        // encrypt json if its an encrypted profile
        if self.encrypted {
            let (key, iv) = password_to_key(args.flag_key.as_slice());
            buffer = encrypt(
                buffer.as_slice(),
                key.as_slice(),
                iv.as_slice()
            ).ok().unwrap();
        }

        // write buffer to file
        file.write(buffer.as_slice())
            .ok()
            .expect(format!("Couldn't write to {}",profile_path.display()).as_slice());
    }

    fn add_item(&mut self, args: &Args) {
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
            drop_to_editor(&"".to_string())
        } else if args.cmd__ {
            stdin().lock().read_to_string().unwrap()
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
            last_touched: strftime("%F %T", &now_utc()).ok().unwrap()
        });
        println!("added");
    }

    fn delete_item(&mut self, id: usize) {
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

    fn edit_item(&mut self, id: usize, args: &Args) {
        let item_pos: usize = self.notes.iter()
            .position(|n| n.id == id)
            .unwrap();
        if !args.arg_title.is_empty() {
            // change title
            self.notes[item_pos].title = args.arg_title.replace("\n", "").to_string();
        } else if args.flag_started || args.flag_urgent || args.flag_none {
            // change status
            if args.flag_started {
                self.notes[item_pos].status = STARTED.to_string();
            } else if args.flag_urgent {
                self.notes[item_pos].status = URGENT.to_string();
            } else if args.flag_none {
                self.notes[item_pos].status = NOSTATUS.to_string();
            }
        } else if !args.flag_b.is_empty() || args.flag_editor || args.cmd__ {
            // change body
            if !args.flag_b.is_empty() {
                self.notes[item_pos].body = args.flag_b.to_string();
            } else if args.flag_editor {
                self.notes[item_pos].body = drop_to_editor(&self.notes[item_pos].body);
            } else if args.cmd__ {
                stdin().lock().read_to_string().unwrap();
            }
        }
        // update last_touched
        self.notes[item_pos].last_touched = strftime("%F %T", &now_utc()).ok().unwrap();
        println!("edited")
    }

    fn stats(&mut self) {
        let mut t = term::stdout().unwrap();
        let no_s = self.notes.iter().filter(|n| n.status == "").count();
        let started_s = self.notes.iter().filter(|n| n.status == "Started").count();
        let urgent_s = self.notes.iter().filter(|n| n.status == "Urgent").count();
        t.attr(Bold).unwrap();
        (write!(t, "encrypted: ")).unwrap();
        t.reset().unwrap();
        (write!(t, "{}\n", self.encrypted)).unwrap();
        t.attr(Bold).unwrap();
        (write!(t, "notes: ")).unwrap();
        t.reset().unwrap();
        (write!(t, "{}\n", self.notes.len())).unwrap();
        t.attr(Bold).unwrap();
        (write!(t, "statuses:")).unwrap();
        t.reset().unwrap();
        (write!(t, " [none: {}, started: {}, urgent: {}]\n", no_s, started_s, urgent_s)).unwrap();
    }

    fn print_header(&mut self, line_format: &LineFormat) {
        let mut t = term::stdout().unwrap();
        let column_seperator: String = repeat(' ').take(line_format.colsep).collect();
        let header_seperator: String = repeat('-').take(line_format.line_width()).collect();
        t.attr(Bold).unwrap();
        (write!(
            t, 
            "{1}{0}{2}{0}{3}{0}{4}\n{5}\n",
            column_seperator,
            format_field(&"id".to_string(), line_format.id_width, false),
            format_field(&"title".to_string(), line_format.title_width, false),
            format_field(&"status".to_string(), line_format.status_width, false),
            format_field(&"last touched".to_string(), line_format.touched_width, false),
            header_seperator
        )).unwrap();
        t.reset().unwrap();
    }

    fn view_item(&mut self, id: usize, args: &Args) {
        let note_pos = self.notes.iter().position(|n| n.id == id).unwrap();
        let mut t = term::stdout().unwrap();

        match args.flag_c {
            true => {
                t.attr(Bold).unwrap();
                (write!(t, "id: ")).unwrap();
                t.reset().unwrap();
                (write!(t, "{}\n", self.notes[note_pos].id)).unwrap();
                t.attr(Bold).unwrap();
                (write!(t, "title: ")).unwrap();
                t.reset().unwrap();
                (write!(t, "{}\n", self.notes[note_pos].title)).unwrap();
                if !self.notes[note_pos].status.is_empty() {
                    t.attr(Bold).unwrap();
                    (write!(t, "status: ")).unwrap();
                    t.reset().unwrap();
                    (write!(t, "{}\n", self.notes[note_pos].status)).unwrap();
                }
                t.attr(Bold).unwrap();
                (write!(t, "last touched: ")).unwrap();
                t.reset().unwrap();
                (write!(t, "{}\n", self.notes[note_pos].last_touched)).unwrap();
            },
            false => {
                t.attr(Bold).unwrap();
                (write!(t, "id\n--\n")).unwrap();
                t.reset().unwrap();
                (write!(t, "{}\n\n", self.notes[note_pos].id)).unwrap();
                t.attr(Bold).unwrap();
                (write!(t, "title\n-----\n")).unwrap();
                t.reset().unwrap();
                (write!(t, "{}\n\n", self.notes[note_pos].title)).unwrap();
                if !self.notes[note_pos].status.is_empty() {
                    t.attr(Bold).unwrap();
                    (write!(t, "status\n------\n")).unwrap();
                    t.reset().unwrap();
                    (write!(t, "{}\n\n", self.notes[note_pos].status)).unwrap();
                }
                t.attr(Bold).unwrap();
                (write!(t, "last touched\n------------\n")).unwrap();
                t.reset().unwrap();
                (write!(t, "{}\n\n", self.notes[note_pos].last_touched)).unwrap();
            }
        };

        // body
        if !self.notes[note_pos].body.is_empty() {
            match args.flag_c {
                true => {
                    t.attr(Bold).unwrap();
                    (write!(t, "body: ")).unwrap();
                    t.reset().unwrap();
                    (write!(t, "{}\n", self.notes[note_pos].body)).unwrap();
                },
                false => {
                    t.attr(Bold).unwrap();
                    (write!(t, "body\n----\n")).unwrap();
                    t.reset().unwrap();
                    (write!(t, "{}\n\n", self.notes[note_pos].body)).unwrap();
                }
            };
        }
    }

    fn list_items(&mut self, args: &Args) {
        if self.notes.len() > 0 {
            let line_format = LineFormat::new(&self.notes, args);
            if !args.flag_c {
                self.print_header(&line_format);
            }
            let list_range = if args.flag_l > 0 {
                args.flag_l
            } else {
                self.notes.len()
            };
            match args.flag_reverse {
                false => {
                    for i in range(0, list_range) {
                        self.notes[i].print(&line_format, args);
                    }
                }, true => {
                    for i in range(0, list_range).rev() {
                        self.notes[i].print(&line_format, args);
                    }
                }
            };
        }
    }

    fn search_items(&mut self, regex_pattern: &str, args: &Args) {
        let re = match Regex::new(regex_pattern) {
            Ok(r) => r,
            Err(e) => panic!("{}", e)
        };
        let notes: Vec<ThecaItem> = match args.flag_body {
            true => self.notes.iter().filter(|n| re.is_match(n.body.as_slice()))
                              .map(|n| n.clone()).collect(),
            false => self.notes.iter().filter(|n| re.is_match(n.title.as_slice()))
                               .map(|n| n.clone()).collect()
        };
        let line_format = LineFormat::new(&notes, args);
        if !args.flag_c {
            self.print_header(&line_format);
        }
        match args.flag_reverse {
            false => {
                for i in range(0, notes.len()) {
                    notes[i].print(&line_format, args);
                    if args.flag_body {
                        // println!("\t{}", notes[i].body);
                        for l in notes[i].body.lines() {
                            println!("\t{}", l);
                        }
                    }
                }
            },
            true => {
                for i in range(0, notes.len()).rev() {
                    notes[i].print(&line_format, args);
                    if args.flag_body {
                        println!("\t{}", notes[i].body);
                    }
                }
            }
        };
    }
}

fn format_field(value: &String, width: usize, truncate: bool) -> String {
    if value.len() > width && width > 3 && truncate {
        format!("{: <1$.1$}...", value, width-3)
    } else {
        format!("{: <1$.1$}", value, width)
    }
}

fn find_profile_folder(args: &Args) -> Path {
    if !args.flag_profiles_folder.is_empty() {
        Path::new(args.flag_profiles_folder.to_string())
    } else {
        match homedir() {
            Some(ref p) => p.join(".theca"),
            None => Path::new(".").join(".theca")
        }
    }
}

fn drop_to_editor(contents: &String) -> String {
    // this could probably be prettyified tbh!

    // setup temporary directory
    let tmpdir = match TempDir::new("theca") {
        Ok(dir) => dir,
        Err(e) => panic!("couldn't create temporary directory: {}", e)
    };
    // setup temporary file to write/read
    let tmppath = tmpdir.path().join(get_time().sec.to_string());
    let mut tmpfile = match File::open_mode(&tmppath, Open, ReadWrite) {
        Ok(f) => f,
        Err(e) => panic!("File error: {}", e)
    };
    tmpfile.write_line(contents.as_slice()).ok().expect("Failed to write line to temp file");
    // we now have a temp file, at `tmppath`, that contains `contents`
    // first we need to know which onqe
    let editor = match getenv("VISUAL") {
        Some(val) => val,
        None => {
            match getenv("EDITOR") {
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

fn get_password() -> String {
    // should really turn off terminal echo...
    print!("Key: ");
    let mut stdin = std::io::stdio::stdin();
    // since this only reads one line of stdin it could still feasibly
    // be used with `-` to set note body?
    stdin.read_line().unwrap().trim().to_string()
}

fn main() {
    let mut args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // is anything stored in the ENV?
    args.check_env();

    if args.flag_encrypted && args.flag_key.is_empty() {
        args.flag_key = get_password();
    }

    // Setup a ThecaProfile struct
    let mut profile = match ThecaProfile::new(&args) {
        Ok(p) => p,
        Err(e) => panic!("{}", e)
    };

    // what root command was used
    if args.cmd_add {
        // add a item
        profile.add_item(&args);
    } else if args.cmd_edit {
        // edit a item
        profile.edit_item(args.arg_id[0], &args);
    } else if args.cmd_del {
        // delete a item
        profile.delete_item(args.arg_id[0]);
    } else if args.flag_v {
        // display theca version
        println!("theca v{}", VERSION);
    } else if args.cmd_search {
        // search for an item
        profile.search_items(args.arg_pattern.as_slice(), &args);
    } else if !args.arg_id.is_empty() {
        // view short item
        profile.view_item(args.arg_id[0], &args);
    } else if args.cmd_info {

        profile.stats();
    } else if !args.cmd_new_profile {
        // this should be the default for nothing
        profile.list_items(&args);
    }

    // save altered profile back to disk
    // this should only be triggered by commands that make transactions to the profile
    if args.cmd_add || args.cmd_edit || args.cmd_del || args.cmd_new_profile {
        profile.save_to_file(&args);
    }
}
