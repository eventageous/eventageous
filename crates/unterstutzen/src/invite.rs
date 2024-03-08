use icalendar::parser;
use mail_parser::{MessageParser, MimeHeaders};

pub struct Invite {}

impl Invite {
    pub fn parse_email(input: &str) -> anyhow::Result<()> {
        let Some(message) = MessageParser::default().parse(input) else {
            anyhow::bail!("could not parse email")
        };
        for attachment in message.attachments() {
            let Some(content_type) = attachment.content_type() else {
                continue;
            };
            if content_type.c_type == "text" && content_type.subtype() == Some("calendar") {
                Self::parse_calendar_invite(attachment.contents())?;
            }
        }
        Ok(())
    }

    fn parse_calendar_invite(ics: &[u8]) -> anyhow::Result<()> {
        let ics = String::from_utf8(ics.to_owned())?;
        let ics = parser::unfold(&ics);
        let calendar = match parser::read_calendar(&ics) {
            Ok(c) => c,
            Err(e) => {
                anyhow::bail!("could not parse calendar: {e}")
            }
        };
        dbg!(calendar);
        Ok(())
    }
}

#[test]
fn test_parse_email() {
    let input = include_str!("../test_data/invite.eml");
    Invite::parse_email(input);
}
