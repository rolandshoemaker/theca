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

use std::old_io::stdio::{stdin};
use std::old_io::{File, Open, ReadWrite,
              TempDir, Command, SeekSet, Read};
use std::old_io::fs::{readdir, PathExtensions};
use time::{get_time};
use std::env::{var_string, home_dir};
use std::old_io::process::{InheritFd};
use term::{stdout};
use term::attr::Attr::{Bold};
use std::old_io::{IoError};
use std::os::errno;
use std::cmp::{Ordering};
use time::{strftime, strptime, at, Tm};
use std::iter::{repeat};
use rustc_serialize::json::{as_pretty_json, decode};

use ::{DATEFMT, DATEFMT_SHORT, ThecaItem, ThecaProfile};
use errors::{ThecaError, GenericError};
use lineformat::{LineFormat};

pub use libc::{
    STDIN_FILENO,
    STDOUT_FILENO,
    STDERR_FILENO
};

// c calls for TIOCGWINSZ
pub mod c {
    extern crate libc;
    pub use self::libc::{
        c_int,
        c_uint,
        c_ushort,
        c_ulong,
        c_uchar,
        STDOUT_FILENO,
        isatty
    };
    use std::mem::zeroed;
    #[derive(Copy)]
    pub struct Winsize {
        pub ws_row: c_ushort,
        pub ws_col: c_ushort
    }
    #[repr(C)]
    #[derive(Copy)]
    pub struct Termios {
        pub c_iflag: c_uint,
        pub c_oflag: c_uint,
        pub c_cflag: c_uint,
        pub c_lflag: c_uint,
        pub c_line: c_uchar,
        pub c_cc: [c_uchar; 32us],
        pub c_ispeed: c_uint,
        pub c_ospeed: c_uint,
    }
    impl Termios {
        pub fn new() -> Termios {
            unsafe {zeroed()}
        }
    }
    pub fn tcgetattr(fd: c_int, termios_p: &mut Termios) -> c_int {
        extern {fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;}
        unsafe {tcgetattr(fd, termios_p as *mut _)}
    }
    pub fn tcsetattr(
        fd: c_int,
        optional_actions: c_int,
        termios_p: &Termios
    ) -> c_int {
        extern {fn tcsetattr(fd: c_int, optional_actions: c_int,
                              termios_p: *const Termios) -> c_int;}
        unsafe {tcsetattr(fd, optional_actions, termios_p as *const _)}
    }
    pub const ECHO:c_uint = 8;
    pub const TCSANOW: c_int = 0;
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
    pub fn istty(fd: c_int) -> bool {
        let isit = unsafe {isatty(fd as i32)};
        isit != 0
    }
}

fn set_term_echo(echo: bool) -> Result<(), ThecaError> {
    let mut t = c::Termios::new();
    try_errno!(c::tcgetattr(STDIN_FILENO, &mut t));
    match echo {
        true => t.c_lflag |= c::ECHO,  // on
        false => t.c_lflag &= !c::ECHO  // off
    };
    try_errno!(c::tcsetattr(STDIN_FILENO, c::TCSANOW, &mut t));
    Ok(())
}

// unsafety wrapper
pub fn termsize() -> usize {
    let ws = unsafe {c::dimensions()};
    if ws.ws_col <= 0 || ws.ws_row <= 0 {
        0
    }
    else {
        ws.ws_col as usize
    }
}

pub fn drop_to_editor(contents: &String) -> Result<String, ThecaError> {
    // setup temporary directory
    let tmpdir = try!(TempDir::new("theca"));
    // setup temporary file to write/read
    let tmppath = tmpdir.path().join(get_time().sec.to_string());
    let mut tmpfile = try!(File::open_mode(&tmppath, Open, ReadWrite));
    try!(tmpfile.write_line(&contents[]));
    let editor = match var_string("VISUAL") {
        Ok(v) => v,
        Err(_) => match var_string("EDITOR") {
            Ok(v) => v,
            Err(_) => specific_fail_str!("neither $VISUAL nor $EDITOR is set.")
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
    match try!(editor_proc).wait().is_ok() {
        true => {
            // finished editing, time to read `tmpfile` for the final output
            // seek to start of `tmpfile`
            try!(tmpfile.seek(0, SeekSet));
            Ok(try!(tmpfile.read_to_string()).trim().to_string())
        }
        false => specific_fail_str!("the editor broke... I think")
    }
}

pub fn get_password() -> Result<String, ThecaError> {
    // should really turn off terminal echo...
    print!("Key: ");
    let tty = c::istty(STDIN_FILENO);
    if tty {try!(set_term_echo(false));}
    let mut stdin = stdin();
    // since this only reads one line of stdin it could still feasibly
    // be used with `-` to set note body?
    let key = try!(stdin.read_line());
    if tty {try!(set_term_echo(true));}
    println!("");
    Ok(key.trim().to_string())
}

pub fn get_yn_input() -> Result<bool, ThecaError> {
    let mut stdin = stdin();
    let mut answer;
    let yes = vec!["y", "Y", "yes", "YES", "Yes"];
    let no = vec!["n", "N", "no", "NO", "No"];
    loop {
        print!("[y/n]# ");
        let mut input = try!(stdin.read_line());
        input = input.trim().to_string();
        match yes.iter().any(|n| &n[] == input) {
            true => {
                answer = true;
                break;
            },
            false => {
                match no.iter().any(|n| &n[] == input) {
                    true => {
                        answer = false;
                        break;
                    },
                    false => ()
                }
            }
        };
        println!("invalid input.");
    }
    Ok(answer)
}

pub fn pretty_line(
    bold: &str,
    plain: &String,
    tty: bool
) -> Result<(), ThecaError> {
    let mut t = match stdout() {
        Some(t) => t,
        None => specific_fail_str!("could not retrieve standard output.")
    };
    if tty {try!(t.attr(Bold));}
    try!(write!(t, "{}", bold.to_string()));
    if tty {try!(t.reset());}
    try!(write!(t, "{}", plain));
    Ok(())
}

pub fn format_field(value: &String, width: usize, truncate: bool) -> String {
    if value.len() > width && width > 3 && truncate {
        format!("{: <1$.1$}...", value, width-3)
    } else {
        format!("{: <1$.1$}", value, width)
    }
}

fn print_header(line_format: &LineFormat) -> Result<(), ThecaError> {
    let mut t = match stdout() {
        Some(t) => t,
        None => specific_fail_str!("could not retrieve standard output.")
    };
    let column_seperator: String = repeat(' ').take(line_format.colsep)
                                              .collect();
    let header_seperator: String = repeat('-').take(line_format.line_width())
                                              .collect();
    let tty = c::istty(STDOUT_FILENO);
    let status = match line_format.status_width == 0 {
        true => "".to_string(),
        false => format_field(
            &"status".to_string(),
            line_format.status_width,
            false
        )+&*column_seperator
    };
    if tty {try!(t.attr(Bold));}
    try!(write!(
                t, 
                "{1}{0}{2}{0}{3}{4}\n{5}\n",
                column_seperator,
                format_field(&"id".to_string(), line_format.id_width, false),
                format_field(
                    &"title".to_string(),
                    line_format.title_width,
                    false
                ),
                status,
                format_field(
                    &"last touched".to_string(),
                    line_format.touched_width,
                    false
                ),
                header_seperator
            ));
    if tty {try!(t.reset());}
    Ok(())
}

pub fn sorted_print(
    notes: &mut Vec<ThecaItem>,
    limit: usize,
    condensed: bool,
    json: bool,
    datesort: bool,
    reverse: bool,
    search_body: bool
) -> Result<(), ThecaError> {
    let line_format = try!(LineFormat::new(notes, condensed, search_body));
    let limit = match limit != 0 && notes.len() >= limit {
        true => limit,
        false => notes.len()
    };
    if !condensed && !json {
        try!(print_header(&line_format));
    }
    if datesort {
        notes.sort_by(|a, b| match cmp_last_touched(
            &*a.last_touched,
            &*b.last_touched
        ) {
            Ok(o) => o,
            Err(_) => a.last_touched.cmp(&b.last_touched) // FIXME(?)
            // Err(_) => Ordering::Equal                  // FIXME(?)
        });
    }
    match json {
        false => {
            if reverse {notes.reverse();}
            for n in notes[0..limit].iter() {
                try!(n.print(&line_format, search_body));
            }
        },
        true => {
            if reverse { notes.reverse(); }
            println!("{}", as_pretty_json(&notes[0..limit].to_vec()))
        }
    };
    
    Ok(())
}

pub fn find_profile_folder(
    profile_folder: &String
) -> Result<Path, ThecaError> {
    if !profile_folder.is_empty() {
        Ok(Path::new(profile_folder.to_string()))
    } else {
        match home_dir() {
            Some(ref p) => Ok(p.join(".theca")),
            None => specific_fail_str!("failed to find your home directory")
        }
    }
}

pub fn parse_last_touched(lt: &str) -> Result<Tm, ThecaError> {
    Ok(at(try!(strptime(lt, DATEFMT)).to_timespec()))
}

pub fn localize_last_touched_string(lt: &str) -> Result<String, ThecaError> {
    let t = try!(parse_last_touched(lt));
    Ok(try!(strftime(DATEFMT_SHORT, &t)))
}

pub fn cmp_last_touched(a: &str, b: &str) -> Result<Ordering, ThecaError> {
    let a_tm = try!(parse_last_touched(a));
    let b_tm = try!(parse_last_touched(b));
    Ok(a_tm.cmp(&b_tm))
}

pub fn validate_profile_from_path(profile_path: &Path) -> (bool, bool) {
    // return (is_a_profile, encrypted(?))
    match profile_path.extension().unwrap() == "json".as_bytes() {
        true => match File::open_mode(
            &profile_path,
            Open,
            Read
        ) {
            Ok(mut f) => {
                let contents_buf = match f.read_to_end() {
                    Ok(c) => c,
                    // nopnopnopppppp
                    Err(_) => return (false, false)
                };
                match String::from_utf8(contents_buf) {
                    Ok(s) => {
                        // well it's a .json and valid utf-8 at least
                        match decode::<ThecaProfile>(&*s) {
                            // yup
                            Ok(_) => return (true, false),
                            // noooooop
                            Err(_) => return (false, false)
                        };
                    },
                    // possibly encrypted
                    Err(_) => return (true, true)
                }
            },
            // nooppp
            Err(_) => return (false, false)
        },
        // noooppp
        false => return (false, false)
    };
}

// this is pretty gross
pub fn path_to_profile_name(profile_path: &Path) -> Result<String, ThecaError> {
    let full_f = try!(String::from_utf8(profile_path.filename().unwrap().to_vec()));
    let ext = try!(String::from_utf8(profile_path.extension().unwrap().to_vec()));
    let just_f = full_f.replace(&(".".to_string()+&ext[])[], "");

    Ok(just_f)
}

pub fn profiles_in_folder(folder: &Path) -> Result<(), ThecaError> {
    if folder.is_dir() {
        let mut contents = try!(readdir(folder));
        contents.sort_by(|a, b| a.cmp(&b));
        println!("# profiles in {}", folder.display());
        for file_path in contents.iter() {
            let is_prof = validate_profile_from_path(file_path);
            if is_prof.0 {
                let mut msg = try!(path_to_profile_name(file_path));
                if is_prof.1 {
                    msg = format!("{} [encrypted?]", msg);
                }
                println!("{}", msg);
            }
        }
    }
    Ok(())
}
