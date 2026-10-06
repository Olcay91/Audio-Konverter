#!/usr/bin/env node
// Setzt die App-Version an allen Stellen, die zusammenpassen müssen:
//   node scripts/set-version.mjs 0.2.0
// Danach committen und den passenden Tag setzen (v0.2.0), siehe README → Releases.

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const version = process.argv[2]?.replace(/^v/, '');
if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error('Aufruf: node scripts/set-version.mjs <version>   (z. B. 0.2.0)');
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function updateJson(file) {
  const path = join(root, file);
  const data = JSON.parse(readFileSync(path, 'utf8'));
  data.version = version;
  writeFileSync(path, JSON.stringify(data, null, 2) + '\n');
  console.log(`${file}: ${version}`);
}

function updateCargo(file) {
  const path = join(root, file);
  const text = readFileSync(path, 'utf8');
  // Nur die erste version-Zeile im [package]-Abschnitt ersetzen
  const updated = text.replace(/(\[package\][^[]*?\nversion\s*=\s*")[^"]*(")/, `$1${version}$2`);
  if (updated === text) {
    console.error(`${file}: keine Versionszeile im [package]-Abschnitt gefunden`);
    process.exit(1);
  }
  writeFileSync(path, updated);
  console.log(`${file}: ${version}`);
}

updateJson('package.json');
updateJson('app/tauri.conf.json');
updateCargo('app/Cargo.toml');

console.log(`\nWeiter mit:\n  git commit -am "Version ${version}"\n  git tag v${version}\n  git push && git push origin v${version}`);
