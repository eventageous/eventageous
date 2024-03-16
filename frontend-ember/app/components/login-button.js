import Component from '@glimmer/component';
import { action } from '@ember/object';

export default class LoginButtonComponent extends Component {

    @action
    redirectToLogin() {
        alert("Not yet! :-)");
        console.log("Not yet! :-)");
    }
}
