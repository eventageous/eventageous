import Service from '@ember/service';
import { tracked } from '@glimmer/tracking';

export default class SessionService extends Service {

    @tracked isLoggedIn = false;

    get isLoggedIn() {
        return this.isLoggedIn;
    }

    async login() {
        let response = await fetch('/auth/login');

        // pretend we're logged in
        this.isLoggedIn = true;
        console.log("Pretend we're logged in");
    }
}
