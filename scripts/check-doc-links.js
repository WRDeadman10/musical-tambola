import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const docsDir = path.resolve(__dirname, '../docs');

if (!fs.existsSync(docsDir)) {
    console.error(`Error: docs directory not found at ${docsDir}`);
    process.exit(1);
}

const mdFiles = fs.readdirSync(docsDir).filter(f => f.endsWith('.md'));
const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;

let brokenLinksCount = 0;
let totalLinksChecked = 0;

for (const file of mdFiles) {
    const filePath = path.join(docsDir, file);
    const content = fs.readFileSync(filePath, 'utf8');

    let match;
    while ((match = linkRegex.exec(content)) !== null) {
        const text = match[1];
        const rawTarget = match[2].trim();

        // Skip anchors, mailto, protocols like http/https/file/conversation
        if (rawTarget.startsWith('#') || rawTarget.includes('://') || rawTarget.startsWith('mailto:')) {
            continue;
        }

        totalLinksChecked++;
        const targetPathOnly = rawTarget.split('#')[0];
        if (!targetPathOnly) continue;

        const resolvedTarget = path.resolve(docsDir, targetPathOnly);
        if (!fs.existsSync(resolvedTarget)) {
            console.error(`Broken link in ${file}: "[${text}](${rawTarget})" -> Target not found: ${targetPathOnly}`);
            brokenLinksCount++;
        }
    }
}

console.log(`\nDoc link check complete: verified ${totalLinksChecked} relative links across ${mdFiles.length} markdown files.`);

if (brokenLinksCount > 0) {
    console.error(`FAIL: Found ${brokenLinksCount} broken relative link(s).`);
    process.exit(1);
} else {
    console.log('SUCCESS: All relative markdown links are valid.');
    process.exit(0);
}
