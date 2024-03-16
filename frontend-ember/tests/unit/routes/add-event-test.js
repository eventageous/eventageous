import { module, test } from 'qunit';
import { setupTest } from 'eventageous-frontend/tests/helpers';

module('Unit | Route | add-event', function (hooks) {
  setupTest(hooks);

  test('it exists', function (assert) {
    let route = this.owner.lookup('route:add-event');
    assert.ok(route);
  });
});
