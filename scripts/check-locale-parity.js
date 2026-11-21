import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const enPath = path.resolve(__dirname, '../src/locales/en.json');
const guPath = path.resolve(__dirname, '../src/locales/gu.json');

if (!fs.existsSync(enPath) || !fs.existsSync(guPath)) {
    console.error('Error: Locale files not found at expected paths.');
    process.exit(1);
}

const en = JSON.parse(fs.readFileSync(enPath, 'utf8'));
const gu = JSON.parse(fs.readFileSync(guPath, 'utf8'));

function flattenKeys(obj, prefix = '') {
    let result = [];
    for (const key of Object.keys(obj)) {
        const fullKey = prefix ? `${prefix}.${key}` : key;
        const val = obj[key];
        if (typeof val === 'object' && val !== null && !Array.isArray(val)) {
            result = result.concat(flattenKeys(val, fullKey));
        } else {
            result.push(fullKey);
        }
    }
    return result;
}

const enKeys = flattenKeys(en);
const guKeys = flattenKeys(gu);

const missingInGu = enKeys.filter(k => !guKeys.includes(k));
const missingInEn = guKeys.filter(k => !enKeys.includes(k));

let hasError = false;

if (missingInGu.length > 0) {
    console.error(`Missing keys in Gujarati locale (gu.json): ${missingInGu.length}`);
    missingInGu.forEach(k => console.error(`  - ${k}`));
    hasError = true;
}

if (missingInEn.length > 0) {
    console.error(`Missing keys in English locale (en.json): ${missingInEn.length}`);
    missingInEn.forEach(k => console.error(`  - ${k}`));
    hasError = true;
}

if (hasError) {
    console.error('\nFAIL: Locale parity check failed.');
    process.exit(1);
} else {
    console.log(`SUCCESS: Full locale parity verified across ${enKeys.length} keys.`);
    process.exit(0);
}
