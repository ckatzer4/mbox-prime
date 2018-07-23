extern crate tui;
extern crate termion;
extern crate email;
extern crate mbox_reader;

use std::convert::From;
use std::path::Path;

use mbox_reader::MboxFile;

use email::MimeMessage;

mod app;
use app::App;

#[derive(Debug)]
enum ParseError {
    FromUtf8Error(std::string::FromUtf8Error),
    NoData,
    MimeParsingError(email::results::ParsingError),
}

impl From<std::string::FromUtf8Error> for ParseError {
    fn from(err: std::string::FromUtf8Error) -> ParseError {
        ParseError::FromUtf8Error(err)
    }
}

impl From<email::results::ParsingError> for ParseError {
    fn from(err: email::results::ParsingError) -> ParseError {
        ParseError::MimeParsingError(err)
    }
}

fn parse_as_mime(mail: mbox_reader::Entry) -> Result<email::MimeMessage, ParseError> {
    let message_bytes = mail
        .message().ok_or(ParseError::NoData)?
        .to_vec();
    let message_string = String::from_utf8_lossy(&message_bytes).to_string();
    Ok(MimeMessage::parse(&message_string)?)
}

fn missing_subject_header() -> email::Header {
    let name: String = "Subject".into();
    let value: String = "MISSING SUBJECT".into();
    email::Header::new(name, value)
}

fn main() {
    // let path = Path::new("./opensuse-factory-2018-06.mbox");
    let path = std::env::args().nth(1).expect("Missing file");
    let path = Path::new(&path);
    let mailbox = MboxFile::from_file(path).unwrap();
    let messages: Vec<_> = mailbox.iter().map(|m| parse_as_mime(m).unwrap()).collect();
    let names: Vec<_> = messages.iter()
        .map(
            // Extract subject or fallback to "MISSING SUBJECT"
            |m| m.headers.get("Subject".into()).unwrap_or(&missing_subject_header())
                .get_value().unwrap()
        )
        .collect();
    let mut my_app = App::new(names, messages);
    my_app.run()
}
