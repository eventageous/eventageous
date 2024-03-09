pub mod calendar_transformer {
    use crate::calendar::AmericanoEvent;
    use crate::calendar::AmericanoEvents;
    use crate::google_calendar::Events;

    pub fn google_to_americano(g_events: Events) -> AmericanoEvents {
        let mut v: Vec<AmericanoEvent> = Vec::with_capacity(g_events.items.capacity());
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

            let event = AmericanoEvent {
                summary: g_event.summary.clone(),
                description: g_event.description.clone(),
                location: g_event.location.clone(),
                creator_email: creator_email,
                creator_name: creator_name,
                start_datetime: start_datetime,
                start_timezone: start_timezone,
                end_datetime: end_datetime,
                end_timezone: end_timezone,
            };
            v.push(event);
        }

        AmericanoEvents { events: v }
    }
}
