import Route from '@ember/routing/route';
import { service } from '@ember/service';

export default class IndexRoute extends Route {
  @service session;

  async model() {
    let response = await fetch('/api/events');
    console.log("Response:");
    console.log(response);

    let { data, authed } = await response.json()

    // Set the session state - this is a simple example, should be done differently
    this.session.loggedIn = authed;

    console.log(data.events);
    return data.events;
  }
}
