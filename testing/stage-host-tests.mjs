// Legacy CLI gates resolve schemas, fixtures and compiled executables relative to the host.
import assert from 'node:assert/strict';
import {appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {dirname, join, resolve} from 'node:path';

const host = process.cwd();
const candidate = resolve(host, '.ci/candidate');
const {id} = JSON.parse(readFileSync(join(candidate, 'lekalo.package.json')));
const manifest = join(candidate, 'host-test-paths.json');
if (existsSync(manifest)) {
  const scopes = JSON.parse(readFileSync(manifest));
  for (const scope of scopes) {
    assert.ok(/^(scripts|tests|adapters|examples)\/[\w./-]+$/.test(scope));
    assert.ok(scope.split('/').every(part => part !== '.' && part !== '..' && part !== ''));
    rmSync(join(host, scope), {recursive: true, force: true});
  }
  const paths = execFileSync('git', ['-C', candidate, 'ls-files', '-z', '--', 'host-tests/'], {encoding:'utf8'}).split('\0').filter(Boolean);
  for (const path of paths) {
    const relative = path.slice('host-tests/'.length);
    assert.ok(scopes.some(scope => relative === scope || relative.startsWith(scope + '/')), 'Unowned test file: ' + relative);
    const destination = join(host, relative);
    mkdirSync(dirname(destination), {recursive: true});
    copyFileSync(join(candidate, path), destination);
  }
}
appendFileSync(process.env.GITHUB_ENV, 'LEKALO_EXTERNAL_TOOLS=' + JSON.stringify({[id]:candidate}) + '\n');
