const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const root = path.resolve(__dirname, '../..');
const source = path.join(root, 'frontend/public/logos/onebrain-logo.png');
const tauriCli = path.join(root, 'node_modules/@tauri-apps/cli/tauri.js');
const icons = path.join(root, 'src-tauri/icons');
const generated = fs.mkdtempSync(path.join(os.tmpdir(), 'onebrain-icons-'));

try {
  execFileSync(process.execPath, [tauriCli, 'icon', source, '-o', generated], {
    cwd: root,
    stdio: 'inherit',
  });
  for (const file of ['icon.png', 'icon.icns', 'icon.ico']) {
    fs.copyFileSync(path.join(generated, file), path.join(icons, file));
  }
  // Tauri generates an RGBA PNG for the bundle on every platform.
  fs.copyFileSync(path.join(generated, 'icon.png'), path.join(icons, 'icon-square.png'));
} finally {
  fs.rmSync(generated, { recursive: true, force: true });
}
