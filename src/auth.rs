use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write}; // Read is used by BufReader and read_to_string
use std::path::Path;
use std::str; // For from_utf8

pub const USERS_FILE_PATH: &str = "users.xml";

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub username: String,
    pub is_admin: bool,
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn prompt_password_hidden(prompt_text: &str) -> io::Result<String> {
    rpassword::prompt_password(prompt_text)
}

pub fn get_confirmed_password(
    prompt_prefix: &str,
) -> io::Result<String> {
    loop {
        let pass1 =
            prompt_password_hidden(&format!("{}: > ", prompt_prefix))?;
        let pass2 = prompt_password_hidden("Confirm Password: > ")?;

        if pass1 == pass2 {
            if pass1.is_empty() {
                println!("Password cannot be empty. Please try again.");
            } else {
                return Ok(pass1);
            }
        } else {
            println!("Passwords do not match. Please try again.");
        }
    }
}

pub fn load_users() -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let path = Path::new(USERS_FILE_PATH);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.trim_text(true);

    let mut users = Vec::new();
    let mut buf = Vec::new();

    let mut current_username_tag: Option<String> = None;
    let mut current_password_hash: Option<String> = None;
    let mut current_is_admin: Option<bool> = None;
    let mut in_password_element = false;
    let mut in_isadmin_element = false;

    loop {
        match xml_reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let tag_name =
                    String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match tag_name.as_str() {
                    "users" => {}
                    "password" => {
                        if current_username_tag.is_some() {
                            in_password_element = true;
                        }
                    }
                    "isadmin" => {
                        if current_username_tag.is_some() {
                            in_isadmin_element = true;
                        }
                    }
                    uname_tag => {
                        current_username_tag = Some(uname_tag.to_string());
                        current_password_hash = None;
                        current_is_admin = None;
                    }
                }
            }
            Event::Text(e) => {
                if current_username_tag.is_some() {
                    // Corrected: unescape to Cow<[u8]>, then convert to String
                    let unescaped_bytes = e.unescape()?;
                    let text_content =
                        unescaped_bytes.to_string();

                    if in_password_element {
                        current_password_hash = Some(text_content);
                    } else if in_isadmin_element {
                        current_is_admin =
                            Some(text_content.eq_ignore_ascii_case("yes"));
                    }
                }
            }
            Event::End(e) => {
                let tag_name =
                    String::from_utf8_lossy(e.name().as_ref()).into_owned();
                match tag_name.as_str() {
                    "password" => in_password_element = false,
                    "isadmin" => in_isadmin_element = false,
                    "users" => {}
                    ended_username_tag => {
                        if Some(ended_username_tag.to_string())
                            == current_username_tag
                        {
                            if let (
                                Some(username),
                                Some(hash),
                                Some(is_admin),
                            ) = (
                                current_username_tag.take(),
                                current_password_hash.take(),
                                current_is_admin.take(),
                            ) {
                                users.push(User {
                                    username,
                                    password_hash: hash,
                                    is_admin,
                                });
                            }
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(users)
}

pub fn save_users(
    users: &[User],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(USERS_FILE_PATH)?;
    let mut xml_writer = Writer::new(BufWriter::new(file));

    xml_writer
        .write_event(Event::Start(BytesStart::new("users")))?;

    for user in users {
        // Corrected: Pass &user.username (which is &str) directly
        xml_writer
            .write_event(Event::Start(BytesStart::new(&user.username)))?;

        xml_writer
            .write_event(Event::Start(BytesStart::new("password")))?;
        xml_writer.write_event(Event::Text(BytesText::new(
            &user.password_hash,
        )))?;
        xml_writer.write_event(Event::End(BytesEnd::new("password")))?;

        xml_writer
            .write_event(Event::Start(BytesStart::new("isadmin")))?;
        xml_writer.write_event(Event::Text(BytesText::new(
            if user.is_admin { "yes" } else { "no" },
        )))?;
        xml_writer.write_event(Event::End(BytesEnd::new("isadmin")))?;

        // Corrected: Pass &user.username (which is &str) directly
        xml_writer.write_event(Event::End(BytesEnd::new(&user.username)))?;
    }

    xml_writer.write_event(Event::End(BytesEnd::new("users")))?;
    xml_writer.into_inner().flush()?;
    Ok(())
}
