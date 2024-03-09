import Component from '@ember/component';
import { htmlSafe } from '@ember/template';

export default class Event extends Component {

    formatLocation(location) {
        const regex = new RegExp('^http(s)?://');
        let link;
        if (location && regex.test(location)) {
            link = '<a href="' + location + '">' + location + '</a>';
        }
        // maybe it's a location?
        if (location && location.includes(',')) {
            let googleQuery = 'https://www.google.com/maps/search/?api=1&query=' + encodeURIComponent(location);
            link = '<a href="' + googleQuery + '">' + location + '</a>';
        }
        if (link) {
            // NOTE: htmlSafe declares we trust this string as safe, it does not MAKE it safe
            return htmlSafe(link);
        }
        return location;
    }

    formatRecurrence(recurrance) {
        if (recurrance) {
            return 'Recurring';
        }
        return 'One-time';
    }

    formatDescription(description) {
        // TODO: stuff
        return description;
    }

    actions = {
        subscribe(event) {
            alert("Not yet! :-)");
            console.log("Not yet! :-)");
        }
    }
}

