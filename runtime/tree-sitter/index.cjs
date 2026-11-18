// Shared bounded syntax evidence for independent Lekalo language packages.
const {readFileSync} = require('node:fs');
const {join} = require('node:path');
const {Parser, Language, Query} = require('./tree-sitter.cjs');

async function evaluate(config, input, directory, extra) {
  await Parser.init({wasmBinary: readFileSync(join(__dirname, 'tree-sitter.wasm'))});
  const groups = new Map(), declarations = [], facts = {};
  try {
    for (const file of input.files) {
      const grammar = config.grammars.find(g => g.extensions.some(ext => file.path.endsWith(ext)));
      if (!grammar) continue;
      let group = groups.get(grammar.file);
      if (!group) {
        const language = await Language.load(readFileSync(join(directory, grammar.file)));
        const parser = new Parser();
        parser.setLanguage(language);
        group = {language, parser, trees: [], declarations: new Query(language, grammar.declarations)};
        groups.set(grammar.file, group);
      }
      const tree = group.parser.parse(file.content);
      if (!tree) throw Error('parse-incomplete');
      group.trees.push({path: file.path, tree});
      if (tree.rootNode.hasError) throw Error('syntax-incomplete');
      for (const capture of group.declarations.captures(tree.rootNode)) {
        declarations.push({path: file.path, kind: capture.name, name: capture.node.text});
        if (declarations.length > 10000) throw Error('declaration-limit');
      }
    }
    for (const pack of input.rules) {
      const values = {};
      for (const check of pack.rules.checks) {
        const queryText = pack.rules.bindings?.[config.id + '.queries/v1']?.[check.id];
        if (queryText !== undefined && pack.rules.requires.includes('source.matches/v1')) {
          const matches = [];
          for (const group of groups.values()) {
            const query = new Query(group.language, queryText);
            try {
              for (const {path, tree} of group.trees) {
                for (const capture of query.captures(tree.rootNode, {matchLimit: 10000})) {
                  matches.push({path, kind: capture.name, name: capture.node.text});
                  if (matches.length > 10000) throw Error('match-limit');
                }
                if (query.didExceedMatchLimit()) throw Error('match-limit');
              }
            } finally { query.delete(); }
          }
          values[check.id] = {matches};
        } else if (pack.rules.requires.length === 1 && pack.rules.requires[0] === 'source.declarations/v1') {
          values[check.id] = {declarations};
        } else if (extra) {
          values[check.id] = extra(pack, check, groups, input.rules);
        } else { throw Error('unsupported-facts'); }
      }
      facts[pack.id] = values;
    }
    return {state: 'complete', facts};
  } finally {
    for (const group of groups.values()) {
      for (const {tree} of group.trees) tree.delete();
      group.declarations.delete();
      group.parser.delete();
    }
  }
}

exports.run = async function run(config, directory, extra) {
  const request = JSON.parse(readFileSync(0, 'utf8'));
  if (request.protocol !== 'lekalo/runtime/v1') throw Error('protocol');
  let result;
  if (request.operation === 'describe') {
    result = {id: config.id, version: config.version, operations: ['check'],
      capabilities: ['source.declarations/v1', 'source.matches/v1', ...(config.capabilities || [])],
      rules_api: ['lekalo/contract-rules/v1']};
  } else if (request.operation === 'check') {
    try { result = await evaluate(config, request.input, directory, extra); }
    catch { result = {state: 'partial', facts: {}}; }
  } else { throw Error('operation'); }
  process.stdout.write(JSON.stringify({protocol: request.protocol, request_id: request.request_id, result}));
};
