import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const commandsDir = path.resolve(__dirname, '../src-tauri/src/commands');

if (!fs.existsSync(commandsDir)) {
    console.error(`Error: Commands directory not found at ${commandsDir}`);
    process.exit(1);
}

const rsFiles = fs.readdirSync(commandsDir).filter(f => f.endsWith('.rs'));
const commands = [];

for (const file of rsFiles) {
    const content = fs.readFileSync(path.join(commandsDir, file), 'utf8');
    const regex = /#\[tauri::command\]\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(([^)]*)\)/g;
    let match;
    while ((match = regex.exec(content)) !== null) {
        const cmdName = match[1];
        const rawArgs = match[2].trim().replace(/\s+/g, ' ');
        commands.push({
            name: cmdName,
            file,
            args: rawArgs || 'none',
        });
    }
}

commands.sort((a, b) => a.name.localeCompare(b.name));

console.log('# Tauri IPC Command Inventory (Auto-Generated)');
console.log(`Total commands detected: ${commands.length}\n`);
console.log('| # | Command | Source File | Rust Arguments |');
console.log('|---|---|---|---|');

commands.forEach((cmd, idx) => {
    console.log(`| ${idx + 1} | \`${cmd.name}\` | \`${cmd.file}\` | \`${cmd.args}\` |`);
});
