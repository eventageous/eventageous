import Service from '@ember/service';
import { tracked } from '@glimmer/tracking';
import { service } from '@ember/service';

export default class SessionService extends Service {
    // TODO: who will set this value?
    @tracked isLoggedIn = false;

    get isLoggedIn() {
        return this.isLoggedIn;
    }

    login() {
        // pretend we're logged in
        this.isLoggedIn = true;
        console.log("Let's login!");
        window.location.href = '/auth/login';
    }
}
