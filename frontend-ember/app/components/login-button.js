import Component from '@glimmer/component';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

export default class LoginButtonComponent extends Component {
    @service session;

    @action
    async redirectToLogin() {
        console.log("Redirecting to login");
        this.session.login();
    }

    get isLoggedIn() {
        return this.session.isLoggedIn;
    }
}
