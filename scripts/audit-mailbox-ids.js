import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const mailboxPath = path.resolve(__dirname, '../docs/Tel_Mailbox/MAILBOX.md');

if (!fs.existsSync(mailboxPath)) {
    console.error(`Error: Mailbox not found at ${mailboxPath}`);
    process.exit(1);
}

const content = fs.readFileSync(mailboxPath, 'utf8');
const regex = /##\s*\[MSG-(\d+)\]/g;

let match;
const seen = new Map();
const ids = [];

while ((match = regex.exec(content)) !== null) {
    const id = parseInt(match[1], 10);
    ids.push(id);
    const count = (seen.get(id) || 0) + 1;
    seen.set(id, count);
}

const duplicates = [];
for (const [id, count] of seen.entries()) {
    if (count > 1) {
        duplicates.push({ id, count });
    }
}

const skipped = [];
for (let i = 0; i < ids.length - 1; i++) {
    const current = ids[i];
    const next = ids[i + 1];
    if (next > current + 1) {
        for (let missing = current + 1; missing < next; missing++) {
            skipped.push(missing);
        }
    }
}

console.log(`Mailbox audit complete: scanned ${ids.length} messages.`);
console.log(`ID range: MSG-${ids[0]} to MSG-${ids[ids.length - 1]}`);

let hasIssues = false;

if (duplicates.length > 0) {
    console.error(`Duplicate MSG IDs found (${duplicates.length}):`);
    duplicates.forEach(d => console.error(`  - MSG-${d.id} (${d.count} times)`));
    hasIssues = true;
} else {
    console.log('Zero duplicate MSG IDs found.');
}

if (skipped.length > 0) {
    console.warn(`Skipped MSG IDs detected (${skipped.length}):`);
    console.warn(`  - [ ${skipped.map(id => `MSG-${id}`).join(', ')} ]`);
} else {
    console.log('Zero skipped MSG IDs found.');
}

process.exit(hasIssues ? 1 : 0);
