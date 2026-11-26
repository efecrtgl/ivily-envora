// Sürüm tutarlılığı: package.json, src-tauri/Cargo.toml ve (verilirse) git etiketi aynı sürümü göstermeli.
// Kullanım: node scripts/check-version.mjs [vX.Y.Z]
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync(new URL('../package.json', import.meta.url), 'utf8')).version;
const cargoToml = readFileSync(new URL('../src-tauri/Cargo.toml', import.meta.url), 'utf8');
const cargo = /^version\s*=\s*"([^"]+)"/m.exec(cargoToml)?.[1];
const conf = JSON.parse(readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8')).version;
const tag = process.argv[2];

const problems = [];
if (pkg !== cargo) problems.push(`package.json (${pkg}) ile Cargo.toml (${cargo}) farklı`);
if (conf !== '../package.json') problems.push(`tauri.conf.json sürümü package.json'a bağlı olmalı (şu an: ${conf})`);
if (tag && tag.replace(/^v/, '') !== pkg) problems.push(`etiket (${tag}) ile uygulama sürümü (${pkg}) farklı`);
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(pkg)) problems.push(`geçersiz sürüm biçimi: ${pkg}`);

if (problems.length) {
  for (const p of problems) console.error(`✗ ${p}`);
  process.exit(1);
}
console.log(`✓ sürüm ${pkg}${tag ? ` (etiket ${tag})` : ''}`);
