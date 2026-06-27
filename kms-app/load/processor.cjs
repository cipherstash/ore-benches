// Artillery processor: generate a unique user payload per virtual request.
// Kept dependency-free (no faker) so the harness installs fast and runs
// deterministically enough for throughput/latency comparison.

let counter = 0;

function genUser(context, _events, done) {
  counter += 1;
  const unique = `${Date.now()}-${counter}-${Math.floor(Math.random() * 1e9)}`;
  context.vars.email = `user-${unique}@example.com`;
  context.vars.name = `Test User ${unique}`;
  return done();
}

module.exports = { genUser };
