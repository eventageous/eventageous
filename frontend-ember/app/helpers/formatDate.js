export default function formatDate(dateString, timezoneString) {

    const date = new Date(dateString);
    // TODO make locale sensitive
    return new Intl.DateTimeFormat('en-US', {
        dateStyle: 'full',
        timeStyle: 'long',
        timeZone: timezoneString,
    }).format(date);
}