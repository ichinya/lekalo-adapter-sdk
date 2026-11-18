// Run a package's own positive and negative fixture against the public host executable.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import test from 'node:test';

const candidate = process.cwd();
const fixture = JSON.parse(readFileSync(join(candidate, 'test/portable-contract.json')));
const manifest = JSON.parse(readFileSync(join(candidate, 'lekalo.package.json')));
const cli = resolve(process.env.LEKALO_BIN);
const local = 'path:' + candidate;

test(manifest.id + ' accepts and rejects its contract fixture', t => {
  const scratch = mkdtempSync(join(tmpdir(), 'lekalo-owner-'));
  const dir = join(scratch, 'project');
  mkdirSync(dir);
  t.after(() => rmSync(scratch, {recursive: true, force: true}));
  const write = (path, value) => writeFileSync(join(dir, path), JSON.stringify(value));
  const env = {...process.env, LEKALO_PACKAGE_HOME: join(scratch, 'cache')};
  delete env.LEKALO_PROJECT;
  delete env.LEKALO_SERVICES;
  function run(operation, expected) {
    const result = spawnSync(cli, ['--json', 'runtime', operation], {cwd: dir, env, encoding: 'utf8', timeout: 180000});
    assert.ifError(result.error);
    assert.equal(result.status, expected, result.stderr || result.stdout);
    return JSON.parse(expected === 0 ? result.stdout : result.stderr);
  }
  const runtime = manifest.kind === 'runtime' ? local : fixture.runtime;
  let rules = manifest.kind === 'rules' ? local : fixture.rules;
  if (typeof rules === 'object') {
    mkdirSync(join(dir, 'policy'));
    write('policy/lekalo.package.json', {schema:'lekalo/package/v1', id:'fixture-rules', version:'1.0.0', kind:'rules', rules:'rules.json', files:['rules.json']});
    write('policy/rules.json', rules);
    rules = 'path:' + join(dir, 'policy');
  }
  assert.equal(typeof runtime, 'string');
  assert.equal(typeof rules, 'string');
  write('lekalo.yaml', {schema:'lekalo/runtime-config/v1',components:{app:{path:'app',runtime,rules:[rules]}}});
  const source = join(dir, 'app', fixture.path);
  mkdirSync(dirname(source), {recursive: true});
  writeFileSync(source, fixture.valid);
  run('lock', 0);
  run('check', 0);
  const pins = JSON.parse(readFileSync(join(dir, 'lekalo.runtime.lock'))).components.app;
  if (manifest.libraries) {
    for (const id of Object.keys(manifest.libraries)) assert.equal(pins.runtime.package.dependencies[id].manifest.kind, 'library');
  }
  writeFileSync(source, fixture.invalid);
  const {diagnostics} = run('check', 1);
  assert.ok(diagnostics.length);
  assert.ok(diagnostics.every(d => d.id === 'runtime.rule-failed' && d.data.component === 'app'));
});
