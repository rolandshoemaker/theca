extern crate serialize;
extern crate time;
extern crate docopt;
use serialize::{Encodable, Encoder, json};
use time::{now_utc, strftime};
use docopt::Docopt;

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

static USAGE: &'static str = "
theca - cli note taking tool

Usage:
    theca new_profile
    theca new_profile <name> [--encrypted]
    theca [options] [-l LIMIT]
    theca [options] <id>
    theca [options] view <id>
    theca [options] add <title> [-sn | -ss | -su] [-b BODY | -]
    theca [options] edit <id> [-sn | -ss | -su]
    theca [options] del <id>
    theca (-h | --help)
    theca --version

Options:
    -h, --help                          Show this screen.
    --version                           Show the version of theca.
    -p PROFILE, --profile PROFILE       Specify non-default profile.
    -c, --condensed                     Use the condensed print format.
    -e, --expanded                      Use the expanded print format.
    --encrypted                         Encrypt new profile.
    -l LIMIT                            Limit listing to LIMIT items.
    -sn                                 No status.
    -ss                                 Started status.
    -su                                 Urgent status.
    -b BODY                             Set BODY of the item.
    -                                   Set body via STDIN.
";

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
                                encoder.emit_struct("ThecaItem", 0, |encoder| {
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

impl ThecaItem {
    fn format_item(&mut self, id: int) {

    }
}

impl <S: Encoder<E>, E> Encodable<S, E> for ThecaProfile {
	fn encode(&self, encoder: &mut S) -> Result<(), E> {
		match *self {
			ThecaProfile{current_id: ref p_current_id, encrypted: ref p_encrypted, notes: ref p_notes} => {
				encoder.emit_struct("ThecaProfile", 0, |encoder| {
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

fn build_profile(enc: bool) -> ThecaProfile {
    ThecaProfile {
        current_id: 0,
        encrypted: enc,
        notes: vec![]
    }
}

fn main() {
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.parse())
                      .unwrap_or_else(|e| e.exit());

    println!("{}", args);

    // some setup function should do check for existing profile / run preceding line to build a new profile from args etcetcetc..,
	let mut profile = build_profile(false);

    profile.add_item("woo".to_string(), STARTED.to_string(), "this is the body".to_string());
    profile.add_item("another woo".to_string(), NOSTATUS.to_string(), "".to_string());
    profile.delete_item(2);
    profile.delete_item(3);
    profile.add_item("another woo".to_string(), URGENT.to_string(), "".to_string());

	println!("profile: {}", json::encode(&profile));
}
