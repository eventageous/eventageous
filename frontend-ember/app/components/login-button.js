import Component from '@glimmer/component';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

export default class LoginButtonComponent extends Component {
    @service session;

    @action
    async redirectToLogin() {
        this.session.login();
    }
}
