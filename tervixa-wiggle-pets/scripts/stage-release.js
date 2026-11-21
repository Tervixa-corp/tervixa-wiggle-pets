// Stages a per-OS release package under releases/<os>/.
// usage: node scripts/stage-release.js windows|linux|macos
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const os = process.argv[2] || process.platform;
const root = path.join(__dirname, "..");
const stage = path.join(root, "releases", os);
fs.mkdirSync(stage, { recursive: true });

const exe = process.platform === "win32" ? ".exe" : "";
const coreSrc = path.join(root, "src-core", "target", "release");
const binSrc = path.join(coreSrc, `wiggle-server${exe}`);
const hookSrc = path.join(coreSrc, `wiggle-hook${exe}`);
if (!fs.existsSync(binSrc)) {
  console.error("core binaries missing — build first");
  process.exit(1);
}

// stage: bin/ + assets/ + src-ui + package.json
const pkg = path.join(stage, "tervixa-wiggle-pets");
fs.rmSync(pkg, { recursive: true, force: true });
fs.mkdirSync(path.join(pkg, "bin"), { recursive: true });
fs.copyFileSync(binSrc, path.join(pkg, "bin", `wiggle-server${exe}`));
fs.copyFileSync(hookSrc, path.join(pkg, "bin", `wiggle-hook${exe}`));
fs.cpSync(path.join(root, "assets"), path.join(pkg, "assets"), { recursive: true });
fs.cpSync(path.join(root, "src-ui"), path.join(pkg, "src-ui"), { recursive: true });
fs.cpSync(path.join(root, "package.json"), path.join(pkg, "package.json"));
fs.copyFileSync(path.join(root, "LICENSE"), path.join(pkg, "LICENSE"));
fs.copyFileSync(path.join(root, "README.md"), path.join(pkg, "README.md"));

// archive
const name = `tervixa-wiggle-pets-${os === "windows" ? "win-x64" : os === "macos" ? "mac-arm64" : "linux-x64"}`;
if (os === "windows") {
  execSync(`powershell -NoProfile -Command "Compress-Archive -Force -Path '${pkg}\\*' -DestinationPath '${stage}\\${name}.zip'"`, { stdio: "inherit" });
} else {
  execSync(`tar -czf "${stage}/${name}.tar.gz" -C "${stage}" tervixa-wiggle-pets`, { stdio: "inherit" });
}
console.log(`staged ${stage}/${name}.zip`);
