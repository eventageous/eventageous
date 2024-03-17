import Route from '@ember/routing/route';
import { service } from '@ember/service';

export default class IndexRoute extends Route {
  @service session;

  async model() {
    let response = await fetch('/api/events');
    console.log("Response:");
    console.log(response);

    let { data, authed, email } = await response.json()

    // Set the session state - this is a simple example, should be done differently
    this.session.loggedIn = authed;
    this.session.userEmail = email;

    console.log(data.events);
    console.log("logged in:" + this.session.loggedIn);
    console.log("email:" + this.session.userEmail);
    return data.events;
  }
}
