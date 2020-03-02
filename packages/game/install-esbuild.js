const { spawn } = require('child_process');
const os = require('os');

if (os.type() === 'Linux')
    spawn("npm", ["install", "esbuild-linux-64"], {stdio: 'inherit'});
else if (os.type() === 'Darwin')
    spawn("npm", ["install", "esbuild-darwin-64"], {stdio: 'inherit'});
else if (os.type() === 'Windows_NT')
    spawn("npm", ["install", "esbuild-windows-64"], {stdio: 'inherit'});
else
    spawn("npm", ["install", "esbuild-wasm"], {stdio: 'inherit'});
