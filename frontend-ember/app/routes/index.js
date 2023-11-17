import Route from '@ember/routing/route';

export default class IndexRoute extends Route {
  async model() {
    let response = await fetch('/api/events.json');

    let { data } = await response.json()

    console.log(data.items);
    return data.items;
  }
}
