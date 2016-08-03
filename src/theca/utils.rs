//  _   _
// | |_| |__   ___  ___ __ _
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// licensed under the MIT license <http://opensource.org/licenses/MIT>
//
// util.rs
//   various utility functions for doings things we need to do.

// std imports
use std::fs::{read_dir, File};
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env::{var, home_dir};
use std::cmp::Ordering;
use std::iter::repeat;
use std::time::UNIX_EPOCH;

// time imports
use time::get_time;
use time::{strftime, strptime, at, Tm};

// term imports
use term::{self, stdout};
use term::Attr::Bold;

// json imports
use rustc_serialize::json::{as_pretty_json, decode};

// tempdir imports
use tempdir::TempDir;

use std::io::stdin;
use std::io::Error as IoError;

// theca imports
use {DATEFMT, DATEFMT_SHORT, Item, Profile, Status, BoolFlags};
use errors::{Result, Error};
use lineformat::LineFormat;

pub use libc::{STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};

// c calls for TIOCGWINSZ
pub mod c {
    extern crate libc;
    pub use self::libc::{c_int, c_uint, c_ushort, c_ulong, c_uchar, STDOUT_FILENO, isatty};
    use std::mem::zeroed;
    use std::default::Default;

    #[derive(Clone, Copy)]
    pub struct Winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Termios {
        pub c_iflag: c_uint,
        pub c_oflag: c_uint,
        pub c_cflag: c_uint,
        pub c_lflag: c_uint,
        pub c_line: c_uchar,
        pub c_cc: [c_uchar; 32usize],
        pub c_ispeed: c_uint,
        pub c_ospeed: c_uint,
    }

    impl Default for Termios {
        fn default() -> Termios {
            unsafe { zeroed() }
        }
    }

    impl Termios {
        pub fn new() -> Termios {
            Default::default()
        }
    }
    pub fn tcgetattr(fd: c_int, termios_p: &mut Termios) -> c_int {
        extern "C" {
            fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
        }
        unsafe { tcgetattr(fd, termios_p as *mut _) }
    }
    pub fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: &Termios) -> c_int {
        extern "C" {
            fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const Termios) -> c_int;
        }
        unsafe { tcsetattr(fd, optional_actions, termios_p as *const _) }
    }
    pub const ECHO: c_uint = 8;
    pub const TCSANOW: c_int = 0;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    static TIOCGWINSZ: c_ulong = 0x5413;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    static TIOCGWINSZ: c_ulong = 0x40087468;
    extern "C" {
        pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    }
    pub unsafe fn dimensions() -> Winsize {
        let mut window: Winsize = zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut window as *mut Winsize);
        window
    }
    pub fn istty(fd: c_int) -> bool {
        let isit = unsafe { isatty(fd as i32) };
        isit != 0
    }
}

fn set_term_echo(echo: bool) -> Result<()> {
    let mut t = c::Termios::new();
    try_errno!(c::tcgetattr(STDIN_FILENO, &mut t));
    if echo {
        t.c_lflag |= c::ECHO;  // on
    } else {
        t.c_lflag &= !c::ECHO;  // off
    };
    try_errno!(c::tcsetattr(STDIN_FILENO, c::TCSANOW, &t));
    Ok(())
}

// unsafety wrapper
pub fn termsize() -> usize {
    let ws = unsafe { c::dimensions() };
    if ws.ws_col == 0 || ws.ws_row == 0 {
        0
    } else {
        ws.ws_col as usize
    }
}

pub fn extract_status(none: bool, started: bool, urgent: bool) -> Result<Option<Status>> {
    match (none, started, urgent) {
        (true, false, false) => Ok(Some(Status::Blank)),
        (false, true, false) => Ok(Some(Status::Started)),
        (false, false, true) => Ok(Some(Status::Urgent)),
        (false, false, false) => Ok(None),
        _ => specific_fail_str!("Can only specify one status"),
    }
}

pub fn drop_to_editor(contents: &str) -> Result<String> {
    // setup temporary directory
    let tmpdir = try!(TempDir::new("theca"));
    // setup temporary file to write/read
    let tmppath = tmpdir.path().join(&format!("{}", get_time().sec)[..]);
    let mut tmpfile = try!(File::create(&tmppath));
    // let mut tmpfile = try!(File::open_mode(&tmppath, Open, ReadWrite));
    try!(tmpfile.write_all(contents.as_bytes()));
    let editor = match var("VISUAL") {
        Ok(v) => v,
        Err(_) => {
            match var("EDITOR") {
                Ok(v) => v,
                Err(_) => return specific_fail_str!("neither $VISUAL nor $EDITOR is set."),
            }
        }
    };
    // lets start `editor` and edit the file at `tmppath`
    // first we need to set STDIN, STDOUT, and STDERR to those that theca is
    // currently using so we can display the editor
    let mut editor_command = Command::new(&editor);
    editor_command.arg(&tmppath.display().to_string());
    editor_command.stdin(Stdio::inherit());
    editor_command.stdout(Stdio::inherit());
    editor_command.stderr(Stdio::inherit());
    let editor_proc = editor_command.spawn();
    if try!(editor_proc).wait().is_ok() {
        // finished editing, time to read `tmpfile` for the final output
        let mut tmpfile = try!(File::open(&tmppath));
        let mut content = String::new();
        try!(tmpfile.read_to_string(&mut content));
        Ok(content)
    } else {
        specific_fail_str!("the editor broke... I think")
    }
}

pub fn get_password() -> Result<String> {
    // should really turn off terminal echo...
    print!("Key: ");
    let tty = c::istty(STDIN_FILENO);
    if tty {
        try!(set_term_echo(false));
    }
    let stdin = stdin();
    let mut key = String::new();
    // since this only reads one line of stdin it could still feasibly
    // be used with `-` to set note body?
    try!(stdin.read_line(&mut key));
    if tty {
        try!(set_term_echo(true));
    }
    println!("");
    Ok(key.trim().to_string())
}

pub fn get_yn_input(message: &str) -> Result<bool> {
    let stdout = match stdout() {
        Some(t) => t,
        None => return specific_fail_str!("could not retrieve standard output."),
    };
    get_yn_input_with_output(stdout, message)
}

pub fn get_yn_input_with_output<W: Write>(mut terminal: Box<term::Terminal<Output=W>>, message: &str) -> Result<bool> {
    try!(write!(terminal, "{}", message));
    try!(terminal.flush());
    let stdin = stdin();
    let answer;
    let yes = vec!["y", "Y", "yes", "YES", "Yes"];
    let no = vec!["n", "N", "no", "NO", "No"];
    loop {
        try!(write!(terminal, "[y/n]# "));
        try!(terminal.flush());
        let mut input = String::new();
        try!(stdin.read_line(&mut input));
        input = input.trim().to_string();
        if yes.iter().any(|n| &n[..] == input) {
            answer = true;
            break;
        } else if no.iter().any(|n| &n[..] == input) {
            answer = false;
            break;
        };
        try!(writeln!(terminal, "invalid input."));
        try!(terminal.flush());
    }
    Ok(answer)
}

pub fn get_stdout() -> Result<Box<term::Terminal<Output=::std::io::Stdout>>> {
    match stdout() {
        Some(t) => Ok(t),
        None => specific_fail_str!("could not retrieve standard output."),
    }
}

pub fn pretty_line(bold: &str, plain: &str, tty: bool) -> Result<()> {
    let mut t = match stdout() {
        Some(t) => t,
        None => return specific_fail_str!("could not retrieve standard output."),
    };
    if tty {
        try!(t.attr(Bold));
    }
    try!(write!(t, "{}", bold.to_string()));
    if tty {
        try!(t.reset());
    }
    try!(write!(t, "{}", plain));
    Ok(())
}

pub fn format_field(value: &str, width: usize, truncate: bool) -> String {
    if value.len() > width && width > 3 && truncate {
        format!("{: <1$.1$}...", value, width - 3)
    } else {
        format!("{: <1$.1$}", value, width)
    }
}

fn print_header(line_format: &LineFormat) -> Result<()> {
    let mut t = match stdout() {
        Some(t) => t,
        None => return specific_fail_str!("could not retrieve standard output."),
    };
    let column_seperator: String = repeat(' ')
                                       .take(line_format.colsep)
                                       .collect();
    let header_seperator: String = repeat('-')
                                       .take(line_format.line_width())
                                       .collect();
    let tty = c::istty(STDOUT_FILENO);
    let status = if line_format.status_width == 0 {
        "".to_string()
    } else {
        format_field(&"status".to_string(), line_format.status_width, false) + &*column_seperator
    };
    if tty {
        try!(t.attr(Bold));
    }
    try!(write!(t,
                "{1}{0}{2}{0}{3}{4}\n{5}\n",
                column_seperator,
                format_field(&"id".to_string(), line_format.id_width, false),
                format_field(&"title".to_string(), line_format.title_width, false),
                status,
                format_field(&"last touched".to_string(),
                             line_format.touched_width,
                             false),
                header_seperator));
    if tty {
        try!(t.reset());
    }
    Ok(())
}

pub fn sorted_print(notes: &mut Vec<Item>,
                    limit: usize,
                    flags: BoolFlags,
                    status: Option<Status>)
                    -> Result<()> {
    let condensed = flags.condensed;
    let json = flags.json;
    let datesort = flags.datesort;
    let reverse = flags.reverse;
    let search_body = flags.search_body;

    // TODO: instead of collecting this, leave as an iterator? using .take(limit) instead of the
    // limit checking code below it?
    if let Some(status) = status {
        notes.retain(|n| n.status == status);
    }
    let limit = if limit != 0 && notes.len() >= limit {
        limit
    } else {
        notes.len()
    };
    if datesort {
        notes.sort_by(|a, b| match cmp_last_touched(&*a.last_touched, &*b.last_touched) {
            Ok(o) => o,
            Err(_) => a.last_touched.cmp(&b.last_touched),
        });
    }

    if reverse {
        notes.reverse();
    }

    if json {
        println!("{}", as_pretty_json(&notes[0..limit].to_vec()))
    } else {
        let line_format = try!(LineFormat::new(&notes[0..limit], condensed, search_body));
        if !condensed && !json {
            try!(print_header(&line_format));
        }
        for n in notes[0..limit].iter() {
            try!(n.print(&line_format, search_body));
        }
    };

    Ok(())
}

pub fn profile_fingerprint<P: AsRef<Path>>(path: P) -> Result<u64> {
    let path = path.as_ref();
    let metadata = try!(path.metadata());
    let modified = try!(metadata.modified());
    let since_epoch = try!(modified.duration_since(UNIX_EPOCH));
    Ok(since_epoch.as_secs())
}

pub fn find_profile_folder(profile_folder: &str) -> Result<PathBuf> {
    if !profile_folder.is_empty() {
        Ok(PathBuf::from(profile_folder))
    } else {
        match home_dir() {
            Some(ref p) => Ok(p.join(".theca")),
            None => specific_fail_str!("failed to find your home directory"),
        }
    }
}

pub fn parse_last_touched(lt: &str) -> Result<Tm> {
    Ok(at(try!(strptime(lt, DATEFMT)).to_timespec()))
}

pub fn localize_last_touched_string(lt: &str) -> Result<String> {
    let t = try!(parse_last_touched(lt));
    Ok(try!(strftime(DATEFMT_SHORT, &t)))
}

pub fn cmp_last_touched(a: &str, b: &str) -> Result<Ordering> {
    let a_tm = try!(parse_last_touched(a));
    let b_tm = try!(parse_last_touched(b));
    Ok(a_tm.cmp(&b_tm))
}

pub fn validate_profile_from_path(profile_path: &PathBuf) -> (bool, bool) {
    // return (is_a_profile, encrypted(?))
    if profile_path.extension().unwrap() == "json" {
        match File::open(profile_path) {
            Ok(mut f) => {
                let mut contents_buf: Vec<u8> = vec![];
                match f.read_to_end(&mut contents_buf) {
                    Ok(c) => c,
                    // nopnopnopppppp
                    Err(_) => return (false, false),
                };
                match String::from_utf8(contents_buf) {
                    Ok(s) => {
                        // well it's a .json and valid utf-8 at least
                        match decode::<Profile>(&*s) {
                            // yup
                            Ok(_) => (true, false),
                            // noooooop
                            Err(_) => (false, false),
                        }
                    }
                    // possibly encrypted
                    Err(_) => (true, true),
                }
            }
            // nooppp
            Err(_) => (false, false),
        }
    } else {
        // noooppp
        (false, false)
    }
}

// this is pretty gross
pub fn path_to_profile_name(profile_path: &PathBuf) -> Result<String> {
    let just_f = profile_path.file_stem().unwrap();

    Ok(just_f.to_str().unwrap().to_string())
}

pub fn profiles_in_folder(folder: &Path) -> Result<()> {
    if folder.is_dir() {
        println!("# profiles in {}", folder.display());
        for file in try!(read_dir(folder)) {
            let file = try!(file);
            let is_prof = validate_profile_from_path(&file.path());
            if is_prof.0 {
                let mut msg = try!(path_to_profile_name(&file.path()));
                if is_prof.1 {
                    msg = format!("{} [encrypted]", msg);
                }
                println!("    {}", msg);
            }
        }
    }
    Ok(())
}
