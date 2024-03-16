import EmberRouter from '@ember/routing/router';
import config from 'eventageous-frontend/config/environment';

export default class Router extends EmberRouter {
  location = config.locationType;
  rootURL = config.rootURL;
}

Router.map(function () {
  this.route('about');
  this.route('new-event');
});
