import { module, test } from 'qunit';
import { setupTest } from 'eventageous-frontend/tests/helpers';

module('Unit | Route | new-event', function (hooks) {
  setupTest(hooks);

  test('it exists', function (assert) {
    let route = this.owner.lookup('route:new-event');
    assert.ok(route);
  });
});
