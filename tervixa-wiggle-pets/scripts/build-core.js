// postinstall helper: builds the Rust core (wiggle-server / wiggle-hook) if a
// working Rust toolchain is available. Fails soft — the Electron UI runs in
// degraded mode without it, and release packages always ship prebuilt binaries.
const { execSync } = require("child_process");
const path = require("path");

const coreDir = path.join(__dirname, "..", "src-core");

function tryBuild(cmd, env) {
  try {
    execSync(cmd, { cwd: coreDir, stdio: "inherit", env: { ...process.env, ...env } });
    return true;
  } catch {
    return false;
  }
}

const isWin = process.platform === "win32";
let ok = false;

if (isWin) {
  // MSVC often broken on this class of machines; try GNU toolchain first.
  const gcc = "C:\\Users\\Remya\\w64devkit\\bin\\gcc.exe";
  const ar = "C:\\Users\\Remya\\w64devkit\\bin\\ar.exe";
  ok =
    tryBuild("cargo +stable-x86_64-pc-windows-gnu build --release", { CC: gcc, AR: ar }) ||
    tryBuild("cargo +stable-x86_64-pc-windows-gnu build --release", {}) ||
    tryBuild("cargo build --release", {});
} else {
  ok = tryBuild("cargo build --release", {});
}

if (!ok) {
  console.warn("[tervixa] rust core build skipped/failed — install rustup and re-run `npm run postinstall`");
  process.exit(0);
}
