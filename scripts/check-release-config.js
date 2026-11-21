import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.resolve(__dirname, '..');

const pkgPath = path.join(rootDir, 'package.json');
const tauriConfPath = path.join(rootDir, 'src-tauri/tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri/Cargo.toml');
const nsisDir = path.join(rootDir, 'src-tauri/target/release/bundle/nsis');

let hasMismatch = false;

// 1. Read package.json
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
const pkgVersion = pkg.version;

// 2. Read tauri.conf.json
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
const tauriVersion = tauriConf.version;

// 3. Read Cargo.toml
const cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');
const cargoMatch = cargoContent.match(/^version\s*=\s*"([^"]+)"/m);
const cargoVersion = cargoMatch ? cargoMatch[1] : null;

console.log('--- Media Tambola Release Configuration Checklist ---');
console.log(`package.json version:     ${pkgVersion}`);
console.log(`tauri.conf.json version:  ${tauriVersion}`);
console.log(`Cargo.toml version:       ${cargoVersion}`);

if (pkgVersion !== tauriVersion) {
    console.error(`MISMATCH: package.json (${pkgVersion}) != tauri.conf.json (${tauriVersion})`);
    hasMismatch = true;
}

if (pkgVersion !== cargoVersion) {
    console.error(`MISMATCH: package.json (${pkgVersion}) != Cargo.toml (${cargoVersion})`);
    hasMismatch = true;
}

// 4. Verify NSIS bundle naming
const expectedInstallerName = `Media Tambola_${pkgVersion}_x64-setup.exe`;
console.log(`Expected installer name:  ${expectedInstallerName}`);

if (fs.existsSync(nsisDir)) {
    const files = fs.readdirSync(nsisDir);
    const installerFound = files.includes(expectedInstallerName);
    if (installerFound) {
        console.log(`NSIS installer file:      FOUND (${expectedInstallerName})`);
    } else {
        console.warn(`NSIS installer file:      NOT FOUND in ${nsisDir}`);
        console.warn(`Available bundles:        [ ${files.join(', ')} ]`);
    }
} else {
    console.warn(`NSIS output directory does not exist: ${nsisDir}`);
}

if (hasMismatch) {
    console.error('\nFAIL: Version configuration mismatch detected.');
    process.exit(1);
} else {
    console.log('\nSUCCESS: All project configurations match release version.');
    process.exit(0);
}
