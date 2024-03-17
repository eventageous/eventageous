import Service from '@ember/service';
import { tracked } from '@glimmer/tracking';
import { service } from '@ember/service';

export default class SessionService extends Service {
    @tracked isLoggedIn = false;
    @tracked userEmail = null;

    login() {
        if (this.isLoggedIn) {
            console.log("Already logged in");
            return;
        }
        console.log("Redirecting to login");
        window.location.href = '/auth/login';
    }
}
