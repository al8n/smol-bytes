import { copyFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(scriptDirectory, '..');

for (const name of ['index', 'shared', 'compact']) {
  copyFileSync(resolve(packageRoot, 'src', `${name}.d.ts`), resolve(packageRoot, 'dist', `${name}.d.ts`));
}
