import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
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

function assertInstalledFiles(packagePath) {
  const missing = expectedFiles.filter((file) => !existsSync(join(packagePath, ...file.split('/'))));
  if (missing.length > 0) {
    throw new Error(`Installed tarball is missing: ${missing.join(', ')}`);
  }
}

function runRuntimeCheck(consumerRoot) {
  const runtimeCheck = join(consumerRoot, 'runtime-check.mjs');
  writeFileSync(
    runtimeCheck,
    `const exportsBySpecifier = {
  'smol-bytes': ['Buffer', 'BytesMut', 'Utf8Buffer', 'Utf8Bytes', 'Utf8BytesMut', 'ByteIterator', 'CharIterator'],
  'smol-bytes/shared': ['Bytes', 'Utf8Bytes'],
  'smol-bytes/compact': ['Bytes', 'Utf8Bytes'],
};

for (const [specifier, expectedExports] of Object.entries(exportsBySpecifier)) {
  const module = await import(specifier);
  for (const name of expectedExports) {
    if (!(name in module)) {
      throw new Error('Runtime export ' + name + ' is missing from ' + specifier);
    }
  }
  console.log('runtime OK: ' + specifier);
}
`,
  );

  try {
    run(process.execPath, [runtimeCheck], { cwd: consumerRoot });
  } catch (error) {
    run(process.execPath, ['--experimental-wasm-modules', runtimeCheck], { cwd: consumerRoot });
  }
}

function runTypeCheck(consumerRoot) {
  const typeCheck = join(consumerRoot, 'consumer.ts');
  writeFileSync(
    typeCheck,
    `import { Buffer, Utf8Bytes } from 'smol-bytes';
import { Bytes as SharedBytes, Utf8Bytes as SharedUtf8Bytes } from 'smol-bytes/shared';
import { Bytes as CompactBytes, Utf8Bytes as CompactUtf8Bytes } from 'smol-bytes/compact';

const values = [Buffer, Utf8Bytes, SharedBytes, SharedUtf8Bytes, CompactBytes, CompactUtf8Bytes];
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
      '--skipLibCheck',
      '--module',
      'ES2020',
      '--moduleResolution',
      'bundler',
      '--target',
      'ES2020',
      typeCheck,
    ],
    { cwd: consumerRoot },
  );
}

function main() {
  const tempRoot = mkdtempSync(join(tmpdir(), 'smol-bytes-package-'));
  try {
    const packDirectory = join(tempRoot, 'pack');
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
    runRuntimeCheck(consumerDirectory);
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
