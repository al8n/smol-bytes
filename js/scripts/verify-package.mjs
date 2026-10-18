import { spawnSync } from 'node:child_process';
import {
  accessSync,
  constants,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const packageRoot = resolve(dirname(scriptPath), '..');
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';

const expectedFiles = [
  'package.json',
  'dist/index.d.ts',
  'dist/index.js',
  'dist/shared.d.ts',
  'dist/shared.js',
  'dist/compact.d.ts',
  'dist/compact.js',
  'dist/runtime.d.ts',
  'dist/runtime.js',
  'pkg/package.json',
  'pkg/smol_bytes.d.ts',
  'pkg/smol_bytes.js',
  'pkg/smol_bytes_bg.js',
  'pkg/smol_bytes_bg.wasm',
  'pkg/smol_bytes_bg.wasm.d.ts',
];

function commandName(command) {
  return command === process.execPath ? process.execPath : command;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: 'utf8',
    env: options.env,
    shell: process.platform === 'win32' && command.endsWith('.cmd'),
    windowsHide: true,
  });

  if (result.error) {
    throw new Error(`Failed to run ${commandName(command)}: ${result.error.message}`);
  }

  if (result.status !== 0) {
    const output = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
    throw new Error(
      `${commandName(command)} ${args.join(' ')} exited with ${result.status ?? 'a signal'}${
        output ? `:\n${output}` : ''
      }`,
    );
  }

  return result.stdout ?? '';
}

function parsePackResult(stdout) {
  try {
    const result = JSON.parse(stdout);
    if (!Array.isArray(result) || result.length === 0 || typeof result[0] !== 'object') {
      throw new Error('npm pack returned an unexpected JSON shape');
    }
    return result[0];
  } catch (error) {
    throw new Error(`Could not parse npm pack output: ${error.message}\n${stdout}`);
  }
}

function assertPackedFiles(packResult) {
  const packedFiles = new Set(
    (packResult.files ?? []).map(({ path }) => path.replaceAll('\\', '/')),
  );
  const missing = expectedFiles.filter((file) => !packedFiles.has(file));
  if (missing.length > 0) {
    throw new Error(`Packed tarball is missing: ${missing.join(', ')}`);
  }
}

function packageInstallPath(consumerRoot, packageName) {
  return join(consumerRoot, 'node_modules', ...packageName.split('/'));
}

function taskTempDirectory() {
  const preferred = process.env.SMOL_BYTES_TMPDIR ?? '/Volumes/ssd/tmp';
  try {
    accessSync(preferred, constants.W_OK);
    return preferred;
  } catch {
    return tmpdir();
  }
}

function assertInstalledFiles(packagePath) {
  const missing = expectedFiles.filter((file) => !existsSync(join(packagePath, ...file.split('/'))));
  if (missing.length > 0) {
    throw new Error(`Installed tarball is missing: ${missing.join(', ')}`);
  }
}

function runRuntimeScript(consumerRoot, name, source) {
  const runtimeCheck = join(consumerRoot, name);
  writeFileSync(runtimeCheck, source);

  try {
    run(process.execPath, [runtimeCheck], { cwd: consumerRoot });
  } catch {
    run(process.execPath, ['--experimental-wasm-modules', runtimeCheck], { cwd: consumerRoot });
  }
}

function runRuntimeCheck(consumerRoot, packagePath) {
  const generatedDeclarations = readFileSync(join(packagePath, 'pkg', 'smol_bytes.d.ts'), 'utf8');
  for (const forbidden of ['__remainingMut', 'unsplitConsume', 'fromStatic']) {
    if (generatedDeclarations.includes(forbidden)) {
      throw new Error(`generated Wasm declarations leak internal name ${forbidden}`);
    }
  }
  const expected = {
    'smol-bytes': ['Buffer', 'ByteIterator', 'Bytes', 'BytesMut', 'CharIterator', 'Utf8Buffer', 'Utf8Bytes', 'Utf8BytesMut'],
    'smol-bytes/shared': ['Bytes', 'Utf8Bytes'],
    'smol-bytes/compact': ['Bytes', 'Utf8Bytes'],
  };
  const assertionSource = `
const expected = ${JSON.stringify(expected)};
function assertExact(module, specifier) {
  const actual = Object.keys(module).sort();
  const wanted = expected[specifier].slice().sort();
  if (actual.length !== wanted.length || actual.some((name, index) => name !== wanted[index])) {
    throw new Error('unexpected exports from ' + specifier + ': ' + actual.join(','));
  }
}
function expectTypeError(fn) {
  try { fn(); } catch (error) { if (error instanceof TypeError) return; throw error; }
  throw new Error('expected TypeError');
}
function expectRangeError(fn) {
  try { fn(); } catch (error) { if (error instanceof RangeError) return; throw error; }
  throw new Error('expected RangeError');
}
function expectIdempotent(value) {
  value.free();
  value.free();
  value[Symbol.dispose]();
  value[Symbol.dispose]();
}
`;

  const isolatedEntrypoints = {
    'runtime-root.mjs': `${assertionSource}
const root = await import('smol-bytes');
assertExact(root, 'smol-bytes');
const raw = root.Bytes.fromUtf8('ok');
const text = root.Utf8Bytes.fromString('é');
if (raw.decodeUtf8() !== 'ok' || [...text][0] !== 'é') throw new Error('root facade was not initialized');
expectIdempotent(raw);
`,
    'runtime-shared.mjs': `${assertionSource}
const shared = await import('smol-bytes/shared');
assertExact(shared, 'smol-bytes/shared');
const raw = shared.Bytes.fromUtf8('ok');
const text = shared.Utf8Bytes.fromString('é');
if ([...raw].length !== 2 || text.byteLength() !== 2) throw new Error('shared facade was not initialized');
expectIdempotent(raw);
expectIdempotent(text);
`,
    'runtime-compact.mjs': `${assertionSource}
const compact = await import('smol-bytes/compact');
assertExact(compact, 'smol-bytes/compact');
const raw = compact.Bytes.fromUtf8('ok');
const text = compact.Utf8Bytes.fromString('é');
if ([...raw].length !== 2 || text.byteLength() !== 2) throw new Error('compact facade was not initialized');
expectIdempotent(raw);
expectIdempotent(text);
`,
  };

  for (const [name, source] of Object.entries(isolatedEntrypoints)) {
    runRuntimeScript(consumerRoot, name, source);
  }

  const orders = [
    ['smol-bytes', 'smol-bytes/shared', 'smol-bytes/compact'],
    ['smol-bytes', 'smol-bytes/compact', 'smol-bytes/shared'],
    ['smol-bytes/shared', 'smol-bytes', 'smol-bytes/compact'],
    ['smol-bytes/shared', 'smol-bytes/compact', 'smol-bytes'],
    ['smol-bytes/compact', 'smol-bytes', 'smol-bytes/shared'],
    ['smol-bytes/compact', 'smol-bytes/shared', 'smol-bytes'],
  ];
  for (const [index, order] of orders.entries()) {
    runRuntimeScript(
      consumerRoot,
      `runtime-order-${index}.mjs`,
      `${assertionSource}
const modules = {};
for (const specifier of ${JSON.stringify(order)}) modules[specifier] = await import(specifier);
for (const [specifier, module] of Object.entries(modules)) assertExact(module, specifier);
const root = modules['smol-bytes'];
const shared = modules['smol-bytes/shared'];
const compact = modules['smol-bytes/compact'];
if (root.Bytes !== shared.Bytes || root.Utf8Bytes !== shared.Utf8Bytes) throw new Error('root/shared aliases lost identity');
if (root.Bytes === compact.Bytes || root.Utf8Bytes === compact.Utf8Bytes) throw new Error('compact classes lost identity separation');
if ([...root.Bytes.fromBytes(new Uint8Array([1]))][0] !== 1) throw new Error('entrypoint patch was not installed');
`,
    );
  }

  runRuntimeScript(
    consumerRoot,
    'runtime-contract.mjs',
    `${assertionSource}
const root = await import('smol-bytes');
const shared = await import('smol-bytes/shared');
const compact = await import('smol-bytes/compact');
assertExact(root, 'smol-bytes');
assertExact(shared, 'smol-bytes/shared');
assertExact(compact, 'smol-bytes/compact');
if (root.Bytes !== shared.Bytes || root.Utf8Bytes !== shared.Utf8Bytes) throw new Error('root/shared aliases lost identity');
if (root.Bytes === compact.Bytes || root.Utf8Bytes === compact.Utf8Bytes) throw new Error('compact classes must remain distinct');
if ('fromString' in root.Bytes || Object.hasOwn(root.Bytes.prototype, 'toString') || 'fromStatic' in root.Bytes) throw new Error('removed raw names are public');
for (const Type of [root.Buffer, root.BytesMut, root.Bytes, compact.Bytes, root.Utf8Buffer, root.Utf8BytesMut, root.Utf8Bytes, compact.Utf8Bytes]) {
  for (const forbidden of ['__remainingMut', 'unsplitConsume', 'fromStatic']) {
    if (forbidden in Type || forbidden in Type.prototype) throw new Error('public class leaks internal name ' + forbidden);
  }
}

for (const Type of [root.Buffer, root.BytesMut, root.Bytes, compact.Bytes]) {
  expectTypeError(() => Type.fromUtf8('\\ud800'));
  expectTypeError(() => Type.fromBytes(new Uint8Array([0xff])).decodeUtf8());
  const bom = Type.fromUtf8('\\uFEFFa');
  if (bom.decodeUtf8() !== '\\uFEFFa' || bom.toBytes()[0] !== 0xef) throw new Error('raw UTF-8 decode lost BOM');
  const short = Type.fromBytes(new Uint8Array([1]));
  expectRangeError(() => short.getU16());
  if (short.remaining() !== 1) throw new Error('fixed getter underflow advanced cursor');
  expectRangeError(() => short.getUint(2));
  if (short.remaining() !== 1) throw new Error('variable getter underflow advanced cursor');
}
const released = root.Bytes.fromUtf8('released');
released.free();
let releasedError;
try {
  released.decodeUtf8();
} catch (error) {
  releasedError = error;
}
if (!releasedError) throw new Error('use-after-free unexpectedly succeeded');
if (releasedError instanceof TypeError || releasedError instanceof RangeError) {
  throw new Error('use-after-free was relabeled as a public validation error');
}

for (const Type of [root.Utf8Buffer, root.Utf8BytesMut, root.Utf8Bytes, compact.Utf8Bytes]) {
  expectTypeError(() => Type.fromString('\\ud800'));
}

for (const [Type, names] of [[root.Buffer, ['putUint', 'putUintLe', 'putUintNe', 'putInt', 'putIntLe', 'putIntNe']], [root.BytesMut, ['putUint', 'putUintLe', 'putUintNe', 'putInt', 'putIntLe', 'putIntNe']]]) {
  for (const name of names) {
    const value = new Type();
    const signed = name.includes('Int') && !name.includes('Uint');
    const invalid = signed ? 128n : 256n;
    expectRangeError(() => value[name](invalid, 1));
    if (value.len() !== 0) throw new Error('width-fit validation mutated value');
    expectRangeError(() => value[name](1n, 0));
    if (value.len() !== 0) throw new Error('zero-width validation mutated value');
  }
}

expectRangeError(() => root.Buffer.fromBytes(new Uint8Array(63)));
expectRangeError(() => root.Utf8Buffer.fromString('x'.repeat(63)));
const fixed = root.Buffer.fromBytes(new Uint8Array(62));
expectRangeError(() => fixed.putU8(1));
if (fixed.len() !== 62) throw new Error('fixed capacity preflight mutated buffer');

const rawSource = root.BytesMut.fromUtf8('raw');
const sharedFrozen = rawSource.freezeShared();
if (!rawSource.isEmpty() || sharedFrozen.decodeUtf8() !== 'raw') throw new Error('shared raw freeze failed');
const rawCopy = sharedFrozen.toMutable();
rawCopy.putU8(33);
if (sharedFrozen.decodeUtf8() !== 'raw' || rawCopy.decodeUtf8() !== 'raw!') throw new Error('raw toMutable independence failed');
const compactSource = root.BytesMut.fromUtf8('compact');
const compactFrozen = compactSource.freezeCompact();
if (!compactSource.isEmpty() || compactFrozen.constructor !== compact.Bytes) throw new Error('compact raw freeze failed');
const rawOther = root.BytesMut.fromUtf8('b');
rawSource.unsplit(rawOther);
if (rawOther.decodeUtf8() !== 'b' || rawSource.decodeUtf8() !== 'b') throw new Error('raw unsplit snapshot failed');

const textSource = root.Utf8BytesMut.fromString('text');
const textFrozen = textSource.freezeShared();
if (!textSource.isEmpty() || textFrozen.toString() !== 'text') throw new Error('shared UTF-8 freeze failed');
const textCopy = textFrozen.toMutable();
textCopy.push('!');
if (textFrozen.toString() !== 'text' || textCopy.toString() !== 'text!') throw new Error('UTF-8 toMutable independence failed');
const compactTextSource = root.Utf8BytesMut.fromString('compact');
const compactText = compactTextSource.freezeCompact();
if (!compactTextSource.isEmpty() || compactText.constructor !== compact.Utf8Bytes) throw new Error('compact UTF-8 freeze failed');
const textOther = root.Utf8BytesMut.fromString('b');
textSource.unsplit(textOther);
if (textOther.toString() !== 'b' || textSource.toString() !== 'b') throw new Error('UTF-8 unsplit snapshot failed');

for (const value of [new root.Buffer(), new root.BytesMut(), new root.Bytes(), new compact.Bytes(), new root.Utf8Buffer(), new root.Utf8BytesMut(), new root.Utf8Bytes(), new compact.Utf8Bytes(), root.Buffer.fromBytes(new Uint8Array([1])).iter(), root.Utf8Buffer.fromString('x').iter()]) {
  expectIdempotent(value);
}
if (typeof root.ByteIterator !== 'function' || typeof root.CharIterator !== 'function') throw new Error('iterator runtime values are missing');
const bytesIterator = root.Buffer.fromBytes(new Uint8Array([1])).iter();
const charsIterator = root.Utf8Buffer.fromString('x').iter();
if (!(bytesIterator instanceof root.ByteIterator) || root.ByteIterator.name !== 'ByteIterator') throw new Error('ByteIterator constructor contract failed');
if (!(charsIterator instanceof root.CharIterator) || root.CharIterator.name !== 'CharIterator') throw new Error('CharIterator constructor contract failed');
`,
  );
}

function runTypeCheck(consumerRoot) {
  const typeCheck = join(consumerRoot, 'consumer.ts');
  writeFileSync(
    typeCheck,
    `import { Buffer, ByteIterator, Bytes, BytesMut, CharIterator, Utf8Bytes } from 'smol-bytes';
import { Bytes as SharedBytes, Utf8Bytes as SharedUtf8Bytes } from 'smol-bytes/shared';
import { Bytes as CompactBytes, Utf8Bytes as CompactUtf8Bytes } from 'smol-bytes/compact';

const raw: IterableIterator<number> = Bytes.fromBytes(new Uint8Array([1])).iter();
const text: IterableIterator<string> = Utf8Bytes.fromString('é').iter();
const byteIteratorValue: typeof ByteIterator = ByteIterator;
const charIteratorValue: typeof CharIterator = CharIterator;
const byteIteratorInstance: ByteIterator = ByteIterator.prototype;
const charIteratorInstance: CharIterator = CharIterator.prototype;
const byteIteratorName: string = ByteIterator.name;
const charIteratorName: string = CharIterator.name;
const byteIteratorCheck: boolean = raw instanceof ByteIterator;
const charIteratorCheck: boolean = text instanceof CharIterator;
// @ts-expect-error iterators are returned by buffer iter() and are not constructible.
new ByteIterator();
// @ts-expect-error iterators are returned by buffer iter() and are not constructible.
new CharIterator();
using disposable = new Buffer();
const mutable = BytesMut.fromUtf8('ok');
const values = [Buffer, ByteIterator, Bytes, BytesMut, CharIterator, Utf8Bytes, SharedBytes, SharedUtf8Bytes, CompactBytes, CompactUtf8Bytes, raw, text, byteIteratorValue, charIteratorValue, byteIteratorInstance, charIteratorInstance, byteIteratorName, charIteratorName, byteIteratorCheck, charIteratorCheck, disposable, mutable];
void values;
`,
  );

  const tsc = join(packageRoot, 'node_modules', 'typescript', 'bin', 'tsc');
  if (!existsSync(tsc)) {
    throw new Error(`Repository TypeScript compiler not found at ${tsc}`);
  }

  run(
    process.execPath,
    [
      tsc,
      '--noEmit',
      '--pretty',
      'false',
      '--strict',
      '--module',
      'ES2022',
      '--moduleResolution',
      'bundler',
      '--target',
      'ES2022',
      '--lib',
      'ES2022,DOM,ESNext.Disposable',
      typeCheck,
    ],
    { cwd: consumerRoot },
  );
}

function main() {
  const tempRoot = mkdtempSync(join(taskTempDirectory(), 'smol-bytes-package-'));
  try {
    const packDirectory = process.env.SMOL_BYTES_PACKAGE_OUTPUT
      ? resolve(packageRoot, process.env.SMOL_BYTES_PACKAGE_OUTPUT)
      : join(tempRoot, 'pack');
    const consumerDirectory = join(tempRoot, 'consumer');
    const npmEnvironment = {
      ...process.env,
      npm_config_audit: 'false',
      npm_config_cache: join(tempRoot, 'npm-cache'),
      npm_config_fund: 'false',
      npm_config_update_notifier: 'false',
    };
    mkdirSync(packDirectory);
    mkdirSync(consumerDirectory);

    const packResult = parsePackResult(
      run(
        npmCommand,
        ['pack', '--json', '--ignore-scripts', '--pack-destination', packDirectory],
        { cwd: packageRoot, env: npmEnvironment },
      ),
    );
    assertPackedFiles(packResult);

    const tarball = join(packDirectory, packResult.filename);
    if (!existsSync(tarball)) {
      throw new Error(`npm pack did not create ${tarball}`);
    }

    writeFileSync(
      join(consumerDirectory, 'package.json'),
      JSON.stringify({ name: 'smol-bytes-package-consumer', private: true }, null, 2),
    );
    run(
      npmCommand,
      [
        'install',
        '--ignore-scripts',
        '--no-package-lock',
        '--no-save',
        '--no-audit',
        '--no-fund',
        tarball,
      ],
      { cwd: consumerDirectory, env: npmEnvironment },
    );

    const installedPackage = packageInstallPath(consumerDirectory, packResult.name);
    assertInstalledFiles(installedPackage);
    runRuntimeCheck(consumerDirectory, installedPackage);
    runTypeCheck(consumerDirectory);

    console.log(`packed package OK: ${relative(packageRoot, tarball)}`);
    console.log('declaration resolution OK: root, shared, compact');
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}
