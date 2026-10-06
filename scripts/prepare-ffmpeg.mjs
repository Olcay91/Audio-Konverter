#!/usr/bin/env node
// Legt ffmpeg und ffprobe so nach app/binaries/, wie Tauri sie für
// "externalBin" erwartet: <name>-<target-triple>[.exe]
//
//   node scripts/prepare-ffmpeg.mjs <pfad/zu/ffmpeg> <pfad/zu/ffprobe> [target-triple]
//
// Ohne target-triple wird das des aktuellen Rechners verwendet (rustc -vV).

import { execSync } from 'node:child_process';
import { chmodSync, copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const [ffmpeg, ffprobe, tripleArg] = process.argv.slice(2);
if (!ffmpeg || !ffprobe) {
  console.error('Aufruf: node scripts/prepare-ffmpeg.mjs <ffmpeg> <ffprobe> [target-triple]');
  process.exit(1);
}

const triple =
  tripleArg ??
  execSync('rustc -vV')
    .toString()
    .split('\n')
    .find((line) => line.startsWith('host:'))
    ?.slice(5)
    .trim();
if (!triple) {
  console.error('Target-Triple konnte nicht ermittelt werden. Bitte als drittes Argument angeben.');
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const outDir = join(root, 'app', 'binaries');
mkdirSync(outDir, { recursive: true });
const ext = triple.includes('windows') ? '.exe' : '';

for (const [name, src] of [['ffmpeg', ffmpeg], ['ffprobe', ffprobe]]) {
  if (!existsSync(src)) {
    console.error(`Nicht gefunden: ${src}`);
    process.exit(1);
  }
  const dest = join(outDir, `${name}-${triple}${ext}`);
  copyFileSync(src, dest);
  if (!ext) chmodSync(dest, 0o755);
  console.log(`${src} -> ${dest}`);
}
