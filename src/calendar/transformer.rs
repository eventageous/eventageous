use crate::calendar::Event;
use crate::calendar::Events;

use super::google_calendar;

pub fn google_to_americano(g_events: google_calendar::Events) -> Events {
    let mut v: Vec<Event> = Vec::with_capacity(g_events.items.capacity());
    for g_event in g_events.items.iter() {
        // Some ad hoc validation
        if g_event.start.is_none()
            || g_event.start.as_ref().unwrap().date_time.is_none()
            || g_event.end.is_none()
            || g_event.end.as_ref().unwrap().date_time.is_none()
            || g_event.creator.email.is_none()
            || g_event.creator.display_name.is_none()
        {
            continue;
        }

        // TODO
        // do some useful transformations, like resolve recurrance dates

        let start_datetime = g_event
            .start
            .as_ref()
            .unwrap()
            .date_time
            .as_ref()
            .unwrap()
            .clone();
        let end_datetime = g_event
            .end
            .as_ref()
            .unwrap()
            .date_time
            .as_ref()
            .unwrap()
            .clone();

        let start_timezone = g_event
            .start
            .as_ref()
            .unwrap()
            .time_zone
            .as_ref()
            .unwrap()
            .clone();
        let end_timezone = g_event
            .end
            .as_ref()
            .unwrap()
            .time_zone
            .as_ref()
            .unwrap()
            .clone();

        let creator_email = g_event.creator.email.as_ref().unwrap().clone();
        let creator_name = g_event.creator.display_name.as_ref().unwrap().clone();

        let recurrance = g_event.recurring_event_id.is_some();

        let event = Event {
            summary: g_event.summary.clone(),
            description: g_event.description.clone(),
            location: g_event.location.clone(),
            creator_email: creator_email,
            creator_name: creator_name,
            start_datetime: start_datetime,
            start_timezone: start_timezone,
            end_datetime: end_datetime,
            end_timezone: end_timezone,
            recurrence: recurrance,
        };
        v.push(event);
    }

    Events { events: v }
}
