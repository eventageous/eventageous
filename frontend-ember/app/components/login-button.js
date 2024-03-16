import Component from '@glimmer/component';
import { action } from '@ember/object';

export default class LoginButtonComponent extends Component {

    @action
    async redirectToLogin() {
        let response = await fetch('/auth/login');
    }
}
