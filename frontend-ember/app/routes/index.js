import Route from '@ember/routing/route';

export default class IndexRoute extends Route {
  async model() {
    let response = await fetch('/api/events');
    console.log("Response:");
    console.log(response);

    let { data } = await response.json()

    console.log(data.events);
    return data.events;
  }
}
