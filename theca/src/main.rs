extern crate serialize;
use serialize::{Encodable, Encoder, json};

static NOSTATUS: &'static str = "";
static STARTED: &'static str = "Started";
static URGENT: &'static str = "Urgent";

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
	fn add_note(&mut self, note: ThecaItem) {
		self.notes.push(note);
	}
}

fn main() {
	let note1 = ThecaItem {
		id: 1,
		title: "wooo".to_string(),
		status: "".to_string(),
		body: "this is the body of this thing".to_string(),
		last_touched: "then".to_string()
	};

    let note2 = ThecaItem {
        id: 2,
        title: "wooo this is another title".to_string(), 
        status: "Urgent".to_string(),
        body: "".to_string(),
        last_touched: "then".to_string()
    };

	let mut profile = ThecaProfile {
		current_id: 2,
		encrypted: false,
		notes: vec![]
	};

	profile.add_note(note1);
	profile.add_note(note2);

	println!("profile: {}", json::encode(&profile));
}
