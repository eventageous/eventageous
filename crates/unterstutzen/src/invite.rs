use mail_parser::{Message, MessageParser, MimeHeaders};
use tracing::info;

use crate::{Creator, End, Event, Start};

#[derive(Default, Debug)]
pub struct CalendarEmail {
    event_requests: Vec<Event>,
}

impl CalendarEmail {
    pub fn parse_email(input: &str) -> anyhow::Result<CalendarEmail> {
        let mut output = CalendarEmail::default();
        let Some(message) = MessageParser::default().parse(input) else {
            anyhow::bail!("could not parse email")
        };
        for attachment in message.attachments() {
            let Some(content_type) = attachment.content_type() else {
                continue;
            };
            if content_type.c_type == "text" && content_type.subtype() == Some("calendar") {
                output.parse_calendar_invite(&message, attachment.contents())?;
            }
        }
        Ok(output)
    }

    fn parse_calendar_invite(&mut self, message: &Message<'_>, ics: &[u8]) -> anyhow::Result<()> {
        let ics = String::from_utf8(ics.to_owned())?;
        let ics = icalendar::parser::unfold(&ics);
        let calendar = match icalendar::parser::read_calendar(&ics) {
            Ok(c) => c,
            Err(e) => {
                anyhow::bail!("could not parse calendar: {e}")
            }
        };

        let Some(method) = calendar.property("METHOD") else {
            anyhow::bail!("Calendar request did not contain METHOD");
        };

        match &*method {
            "REQUEST" => self.parse_calendar_request(message, calendar),

            _ => {
                anyhow::bail!("Calendar request with unknown method: `{method}`");
            }
        }
    }

    fn parse_calendar_request(
        &mut self,
        message: &Message<'_>,
        request: icalendar::parser::Calendar,
    ) -> anyhow::Result<()> {
        // This is a "request to add to the calendar". We will accept it.
        for component in &request.components {
            match component.name.as_str() {
                "VEVENT" => {
                    let event = self.event_from_request(message, component)?;
                    self.event_requests.push(event);
                }
                _ => {
                    info!("unexpected calendar component type, ignoring: {component:?}");
                }
            }
        }

        Ok(())
    }

    fn event_from_request(
        &mut self,
        message: &Message<'_>,
        event: &icalendar::parser::Component<'_>,
    ) -> anyhow::Result<Event> {
        let summary = event.property("SUMMARY").unwrap_or_default();
        let description = event.property("DESCRIPTION");
        let location = event.property("LOCATION");
        let creator = message
            .from()
            .and_then(|address| {
                let email = address.first()?;
                Some(Creator {
                    email: email.address().map(|s| s.to_string()),
                    display_name: email.name().map(|s| s.to_string()),
                })
            })
            .unwrap_or_default();
        let start = event.property("DTSTART").map(|s| Start {
            date_time: Some(s),
            date: None,
            time_zone: None,
        });
        let end = event.property("DTEND").map(|s| End {
            date_time: Some(s),
            date: None,
            time_zone: None,
        });

        Ok(Event {
            summary: summary,
            description: description,
            location: location,
            start,
            end,
            creator,
        })
    }
}

#[test]
#[ignore = "FIXME"]
fn test_parse_email() {
    let input = include_str!("../test_data/invite.eml");
    let calendar = CalendarEmail::parse_email(input).unwrap();
    expect_test::expect![""].assert_debug_eq(&calendar);
}

trait CalendarExt {
    fn property(&self, name: &str) -> Option<String>;
}

impl CalendarExt for icalendar::parser::Calendar<'_> {
    fn property(&self, name: &str) -> Option<String> {
        for property in &self.properties {
            if property.name == name {
                return Some(property.val.as_str().to_string());
            }
        }
        None
    }
}

trait ComponentExt {
    fn property(&self, name: &str) -> Option<String>;
}

impl ComponentExt for icalendar::parser::Component<'_> {
    fn property(&self, name: &str) -> Option<String> {
        for property in &self.properties {
            if property.name == name {
                return Some(property.val.as_str().to_string());
            }
        }
        None
    }
}
