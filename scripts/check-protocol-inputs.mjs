import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve, dirname} from 'node:path';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const lock = JSON.parse(readFileSync(resolve(root, 'protocol-inputs.lock.json')));
for (const [file, expected] of Object.entries(lock.files)) {
  const actual = 'sha256:' + createHash('sha256').update(readFileSync(resolve(root, file))).digest('hex');
  if (actual !== expected) throw new Error(`Protocol input changed without a reviewed pin: ${file}`);
}
console.log(JSON.stringify({ok:true, gate:'protocol-inputs', files:Object.keys(lock.files).length}));
