extern crate serialize;
extern crate time;
extern crate docopt;
use serialize::{Encodable, Decodable, Encoder, json};
use time::{now_utc, strftime};
use docopt::Docopt;
use std::os;
use std::io::fs::PathExtensions;
use std::io::{File, Truncate, Write};

mod c {
    extern crate libc;
    pub use self::libc::{
        c_int,
        c_ushort,
        c_ulong,
        STDOUT_FILENO,
    };
    use std::mem::zeroed;
    pub struct winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort,
    }
    #[cfg(any(target_os = "linux", target_os = "android"))]
    static TIOCGWINSZ: c_ulong = 0x5413;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    static TIOCGWINSZ: c_ulong = 0x40087468;
    extern {
        pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }
    pub unsafe fn dimensions() -> winsize {
        let mut window: winsize = zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window as *mut winsize);
        window
    }
}

fn termsize() -> Option<(uint, uint)> {
    let ws = unsafe { c::dimensions() };

    if ws.ws_col == 0 || ws.ws_row == 0 {
        None
    }
    else {
        Some((ws.ws_col as uint, ws.ws_row as uint))
    }

}

static USAGE: &'static str = "
theca - cli note taking tool

Usage:
    theca new-profile
    theca new-profile <name> [--encrypted]
    theca [options] [-c|-e] [-l LIMIT]
    theca [options] [-c|-e] <id>
    theca [options] [-c|-e] view <id>
    theca [options] add <title> [--started|--urgent] [-b BODY|--editor|-]
    theca [options] edit <id> [--started|--urgent|--none] [-b BODY|--editor|-]
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

#[deriving(Decodable, Show)]
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
    flag_l: Vec<int>,
    arg_title: String,
    flag_started: bool,
    flag_urgent: bool,
    flag_none: bool,
    flag_b: Vec<String>,
    flag_editor: bool,
    cmd__: bool,
    arg_id: Vec<int>,
    flag_h: bool,
    flag_v: bool
}

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

// keep static defaults here to initalize LineFormat?
static COLSEP: uint = 3;

pub struct LineFormat {
    colsep: int,
    id_width: int,
    title_width: int,
    status_width: int,
    touched_width: int
}

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

impl ThecaItem {
    fn decrypt(&mut self, key: &str) {
    }
}

#[deriving(Decodable)]
pub struct ThecaProfile {
    current_id: int,
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
                    last_touched: strftime("%FT%T", &now_utc()).ok().unwrap() // no time...?
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

    fn print_items(&mut self, args: &Args) {
        // SERIOUS SERIOUS optimization, this is awful...
        let (mut id_width, mut title_width, mut status_width, mut touched_width) = (0, 0, 0, 0);
        for i in range(0, self.notes.len()) {
            id_width = if self.notes[i].id.to_string().len() > id_width {self.notes[i].id.to_string().len()} else {id_width};
            if self.notes[i].body.len() > 0 {
                title_width = if self.notes[i].title.len()+4 > title_width {self.notes[i].title.len()+4} else {title_width};
            } else {
                title_width = if self.notes[i].title.len() > title_width {self.notes[i].title.len()} else {title_width};
            }
            status_width = if self.notes[i].status.len() > status_width {self.notes[i].status.len()} else {status_width};
            touched_width = if self.notes[i].last_touched.len() > touched_width {self.notes[i].last_touched.len()} else {touched_width};
        }
        // println!("{} {} {} {}", id_width, title_width, status_width, touched_width);
        let total_width = id_width+title_width+status_width+touched_width+(3*COLSEP);

        let (width, height) = match termsize() {
            None => panic!(),
            Some((width, height)) => (width, height),
        };

        // println!("{} {}", total_width, width);
        
        // if total_width > width {
        //     let new_width = (total_width / width) as f64;
        //     println!("{}", new_width);
        //     id_width = ((id_width as f64) * new_width) as uint;
        //     title_width = ((title_width as f64) * new_width) as uint;
        //     status_width = ((status_width as f64) * new_width) as uint;
        //     touched_width = ((touched_width as f64) * new_width) as uint;
        // }

        if args.flag_e {
            print!("{}", format_field(&"id".to_string(), id_width));
            print!("{}", " ".repeat(COLSEP));
            print!("{}", format_field(&"title".to_string(), title_width));
            print!("{}", " ".repeat(COLSEP));
            print!("{}", format_field(&"status".to_string(), status_width));
            print!("{}", " ".repeat(COLSEP));
            print!("{}", format_field(&"last touched".to_string(), touched_width));
            print!("\n");
            println!("{}", "-".repeat(total_width));
        }

        for i in range(0, self.notes.len()) {
            print!("{}", format_field(&self.notes[i].id.to_string(), id_width));
            print!("{}", " ".repeat(COLSEP));
            if self.notes[i].body.len() > 0 {
                print!("(+) {}", format_field(&self.notes[i].title, title_width-4));
            } else {
                print!("{}", format_field(&self.notes[i].title, title_width));
            }
            print!("{}", " ".repeat(COLSEP));
            print!("{}", format_field(&self.notes[i].status, status_width));
            print!("{}", " ".repeat(COLSEP));
            print!("{}", format_field(&self.notes[i].last_touched, touched_width));
            print!("\n");
        }
    }

    fn print_header(&mut self) {
    }

    fn view_item(&mut self, id: int) {
        let item_pos: uint = self.notes.iter()
            .position(|n| n.id == id).unwrap();
        
    }

    fn list_items(&mut self, args: &Args) {
        self.print_items(args);
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
    if value.len() > width {
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
    if args.cmd_view {
        profile.view_item(args.arg_id[0]);
    } else if args.cmd_add {
        let title = args.arg_title.to_string();
        let status = if args.flag_started {STARTED.to_string()} else if args.flag_urgent {URGENT.to_string()} else {NOSTATUS.to_string()};
        let body = if !args.flag_b.is_empty() {args.flag_b[0].to_string()} else {"".to_string()};
        profile.add_item(title, status, body);
    } else if args.cmd_edit {

    } else if args.cmd_del {
        let id = args.arg_id[0];
        profile.delete_item(id);
    } else if args.flag_v {
        println!("VERSION YO");
    } else if !args.cmd_new_profile {
        // this should be the default for nothing
        profile.list_items(&args);
    }

    // save altered profile back to disk
    // this should only be triggered by commands that commit transactions
    if args.cmd_add || args.cmd_edit || args.cmd_del || args.cmd_new_profile {
        profile.save_to_file(&args);
    }

    // profile.add_item("another woo".to_string(), NOSTATUS.to_string(), "".to_string());
    // profile.delete_item(2);
    // profile.delete_item(3);
    // profile.add_item("another woo".to_string(), URGENT.to_string(), "".to_string());
}
