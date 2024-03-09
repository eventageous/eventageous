import Component from '@glimmer/component';
import { htmlSafe } from '@ember/template';

export default class Event extends Component {

    formatLocation(location) {
        const regex = new RegExp('^http(s)?://');
        if (location && regex.test(location)) {
            const link = '<a href="' + location + '">' + location + '</a>';
            // NOTE: htmlSafe declares we trust this string as safe, it does not MAKE it safe
            return htmlSafe(link);
        }
        return location;
    }

    formatDescription(description) {
        // TODO: stuff
        return description;
    }

}