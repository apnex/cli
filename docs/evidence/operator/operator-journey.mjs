import assert from 'node:assert/strict';
import { spawn, execFile } from 'node:child_process';
import fs from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { promisify } from 'node:util';

const execute = promisify(execFile);
const [binaryArgument, agpArgument, outputArgument] = process.argv.slice(2);
assert.ok(binaryArgument && agpArgument && outputArgument, 'binary, AGP repository, and fresh output directory are required');
const binary = resolve(binaryArgument);
const root = resolve(agpArgument);
const output = resolve(outputArgument);
await fs.mkdir(output, { recursive: false });
const config = join(output, 'management.json');
const environment = { ...process.env, AGP_MANAGEMENT_URL: '', XDG_CONFIG_HOME: join(output, 'config-home'), PATH: '' };
const example = spawn(process.execPath, ['examples/loopback-star/example.mjs', '--persist'], {
  cwd: root,
  env: { ...process.env, AGP_LOOPBACK_HUB_MANAGEMENT_PORT: '0', AGP_LOOPBACK_ALPHA_MANAGEMENT_PORT: '0', AGP_LOOPBACK_BETA_MANAGEMENT_PORT: '0' },
  stdio: ['ignore', 'pipe', 'pipe'],
});
let exampleOutput = '', exampleError = '';
example.stderr.on('data', bytes => { exampleError += bytes; });
const exited = new Promise(resolve => example.once('exit', (code, signal) => resolve({ code, signal })));
const records = [];
try {
  const ready = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(Error('Example readiness deadline: ' + exampleError)), 15000);
    example.stdout.on('data', bytes => {
      exampleOutput += bytes;
      const line = exampleOutput.split('\n').find(line => line.startsWith('AGP_LOOPBACK_TOPOLOGY_READY '));
      if (line) { clearTimeout(timer); resolve(JSON.parse(line.slice('AGP_LOOPBACK_TOPOLOGY_READY '.length))); }
    });
    example.once('exit', code => { clearTimeout(timer); reject(Error('Example exited ' + code + ': ' + exampleError)); });
  });
  const hub = ready.nodes.hub.managementUrl;
  const alpha = ready.nodes.alpha.managementUrl;
  async function command(name, args, check) {
    const result = await execute(binary, ['--config', config, ...args], { cwd: root, env: environment, timeout: 15000, maxBuffer: 8 * 1024 * 1024 });
    await fs.writeFile(join(output, name + '.txt'), result.stdout);
    assert.equal(result.stderr, '');
    check?.(result.stdout);
    records.push({ name, arguments: args, exit: 0, bytes: Buffer.byteLength(result.stdout) });
    return result.stdout;
  }
  const help = await command('help', ['--help'], text => {
    assert.ok(text.includes('management set <url>'));
    assert.ok(!text.includes('connected:json-http-get-v1'));
    assert.ok(!text.includes('connections.list'));
  });
  await command('tree', ['tree']);
  await command('detailed-help', ['help', '--all'], text => assert.ok(text.includes('connections.list')));
  await command('unconfigured-management', ['management'], text => assert.ok(text.includes('Endpoint: not configured')));
  const quote = value => "'" + value.replaceAll("'", "'\\''") + "'";
  const terminal = spawn('/usr/bin/script', ['-qefc', `/usr/bin/stty cols 120 rows 30; exec ${quote(binary)} --config ${quote(config)}`, '/dev/null'], {
    cwd: root, env: { ...environment, TERM: 'xterm-256color' }, stdio: ['pipe', 'pipe', 'pipe'], detached: true,
  });
  const steps = [
    ['agp [/]', '?\r'],
    ['Structure: tree    Leave: exit', '/\r'],
    ['Use help to list contexts and commands.', 'ls\r'],
    ['Structure: tree    Leave: exit', 'show\r'],
    ['Structure: tree    Leave: exit', 'routes\r'],
    ['Commands: ls, show. Use help for details.', 'show\r'],
    ['Use management set <url>, then retry the command.', `management set ${hub}\r`],
    ['Use management save to keep this selection.', 'management save\r'],
    ['Saved management settings for future launches.', '/\r'],
    ['Use help to list contexts and commands.', 'connections\r'],
    ['Commands: ls, show. Use help for details.', 'show\r'],
    ['Established', '?\r'],
    ['Structure: tree    Leave: exit', '/\r'],
    ['Use help to list contexts and commands.', 'tree\r'],
    ['management', 'exit\r'],
  ];
  let transcript = '', pending = '', stage = 0;
  const terminalResult = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => { process.kill(-terminal.pid, 'SIGTERM'); reject(Error('Terminal deadline at stage ' + stage)); }, 30000);
    terminal.stdout.on('data', bytes => {
      const text = bytes.toString(); transcript += text; pending += text;
      for (const match of text.matchAll(/\x1b\[6n/g)) terminal.stdin.write('\x1b[1;1R');
      while (stage < steps.length) {
        const [needle, input] = steps[stage]; const index = pending.indexOf(needle);
        if (index < 0) break;
        pending = pending.slice(index + needle.length); terminal.stdin.write(input); stage++;
      }
    });
    terminal.once('error', error => { clearTimeout(timer); reject(error); });
    terminal.once('close', (code, signal) => { clearTimeout(timer); resolve({ code, signal, stage }); });
  }).finally(() => fs.writeFile(join(output, 'operator-terminal.raw'), transcript));
  assert.equal(terminalResult.code, 2, 'the deliberately unconfigured read remains an exit failure');
  assert.equal(terminalResult.stage, steps.length);
  assert.ok(transcript.includes('Established'));
  assert.ok(!transcript.includes('INVALID_RUN_COMMAND'));
  assert.equal(JSON.parse(await fs.readFile(config, 'utf8')).endpoint, hub);
  await command('reopened-health', ['health', 'show', '--json'], text => {
    const health = JSON.parse(text); assert.equal(health.meta.nodeId, 'hub'); assert.equal(health.data.ready, true);
  });
  await command('saved-management', ['management'], text => assert.ok(text.includes('Selection source: saved')));
  await command('switch-node', ['management', 'set', alpha, '--save']);
  await command('switched-health', ['health', 'show', '--json'], text => {
    const health = JSON.parse(text); assert.equal(health.meta.nodeId, 'leaf.alpha'); assert.equal(health.data.ready, true);
  });
  await command('clear-saved', ['management', 'clear', '--save']);
  let clearedError;
  await assert.rejects(execute(binary, ['--config', config, 'health', 'show'], { cwd: root, env: environment, timeout: 15000 }), error => {
    clearedError = error.stderr;
    assert.equal(error.code, 2); assert.equal(error.stdout, '');
    assert.equal(error.stderr, 'No management endpoint configured.\nUse management set <url>, then retry the command.\n');
    return true;
  });
  await fs.writeFile(join(output, 'cleared-read.txt'), clearedError);
  const report = { binary, endpoints: { hub, alpha }, cases: records, rootHelpLines: help.trimEnd().split('\n').length, terminal: terminalResult, clearedReadExit: 2, userDefaultWritten: false };
  await fs.writeFile(join(output, 'measurements.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report));
} finally {
  example.kill('SIGTERM');
  const stopped = await exited;
  await fs.writeFile(join(output, 'example.stdout'), exampleOutput);
  await fs.writeFile(join(output, 'example.stderr'), exampleError);
  await fs.writeFile(join(output, 'example-exit.json'), JSON.stringify(stopped) + '\n');
  assert.equal(stopped.code, 0, JSON.stringify(stopped));
}
