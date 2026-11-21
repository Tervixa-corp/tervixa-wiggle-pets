// TERVIXA WIGGLE PETS — Electron main process.
// Owns the transparent always-on-top pet overlay, the Control Center window,
// the tray icon and the Alt+Space quick-command popover.

const { app, BrowserWindow, Tray, Menu, globalShortcut, ipcMain, screen, nativeImage } = require("electron");
const path = require("path");
const fs = require("fs");
const { spawn, spawnSync } = require("child_process");
const http = require("http");

const SERVER_PORT = 23340;
const API = `http://127.0.0.1:${SERVER_PORT}`;

let overlayWin = null;
let dashWin = null;
let chatWin = null;
let tray = null;
let serverProc = null;

// ---------------------------------------------------------------------------
// Rust core daemon lifecycle
// ---------------------------------------------------------------------------

function coreBin() {
  const exe = process.platform === "win32" ? "wiggle-server.exe" : "wiggle-server";
  const candidates = [
    path.join(__dirname, "..", "bin", exe),
    path.join(__dirname, "..", "src-core", "target", "release", exe),
    path.join(__dirname, "..", "..", "src-core", "target", "release", exe),
    path.join(process.resourcesPath || "", exe),
  ];
  return candidates.find((c) => fs.existsSync(c));
}

function apiGet(p) {
  return new Promise((resolve) => {
    http
      .get(`${API}${p}`, (res) => {
        let d = "";
        res.on("data", (c) => (d += c));
        res.on("end", () => {
          try { resolve(JSON.parse(d)); } catch { resolve(null); }
        });
      })
      .on("error", () => resolve(null));
  });
}

function apiPost(p, body) {
  return new Promise((resolve) => {
    const data = JSON.stringify(body || {});
    const req = http.request(
      `${API}${p}`,
      { method: "POST", headers: { "content-type": "application/json", "content-length": Buffer.byteLength(data) } },
      (res) => {
        let d = "";
        res.on("data", (c) => (d += c));
        res.on("end", () => {
          try { resolve(JSON.parse(d)); } catch { resolve(null); }
        });
      }
    );
    req.on("error", () => resolve(null));
    req.write(data);
    req.end();
  });
}

function ensureServer() {
  return apiGet("/health").then((h) => {
    if (h && h.ok) return true;
    const bin = coreBin();
    if (!bin) {
      console.warn("[tervixa] wiggle-server binary not found — UI running in degraded mode");
      return false;
    }
    serverProc = spawn(bin, ["--port", String(SERVER_PORT)], { stdio: "ignore", detached: false });
    return true;
  });
}

// ---------------------------------------------------------------------------
// Windows
// ---------------------------------------------------------------------------

function petAssetsDir() {
  return path.join(__dirname, "..", "assets", "pets");
}

function createOverlay() {
  const wa = screen.getPrimaryDisplay().workArea;
  overlayWin = new BrowserWindow({
    x: wa.x,
    y: wa.y,
    width: wa.width,
    height: wa.height,
    transparent: true,
    frame: false,
    hasShadow: false,
    resizable: false,
    movable: false,
    skipTaskbar: true,
    alwaysOnTop: true,
    fullscreenable: false,
    focusable: true,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });
  overlayWin.setAlwaysOnTop(true, "screen-saver");
  overlayWin.loadFile(path.join(__dirname, "overlay", "index.html"));
  overlayWin.once("ready-to-show", () => overlayWin.show());
  overlayWin.on("closed", () => (overlayWin = null));
}

function createDashboard() {
  if (dashWin) {
    dashWin.focus();
    return;
  }
  dashWin = new BrowserWindow({
    width: 980,
    height: 660,
    minWidth: 760,
    minHeight: 520,
    backgroundColor: "#0A0A0A",
    autoHideMenuBar: true,
    title: "Tervixa Wiggle Pets — Control Center",
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });
  dashWin.loadFile(path.join(__dirname, "dashboard", "index.html"));
  dashWin.on("closed", () => (dashWin = null));
}

function createChat() {
  if (chatWin) {
    chatWin.focus();
    return;
  }
  const wa = screen.getPrimaryDisplay().workArea;
  chatWin = new BrowserWindow({
    x: wa.x + wa.width - 400,
    y: wa.y + wa.height - 420,
    width: 380,
    height: 380,
    frame: false,
    transparent: false,
    backgroundColor: "#0A0A0A",
    skipTaskbar: true,
    alwaysOnTop: true,
    autoHideMenuBar: true,
    resizable: true,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });
  chatWin.loadFile(path.join(__dirname, "bubbles", "chat.html"));
  chatWin.on("closed", () => (chatWin = null));
}

// ---------------------------------------------------------------------------
// Tray
// ---------------------------------------------------------------------------

function trayIcon() {
  // 16x16 mono cloud drawn into a nativeImage (no binary asset needed).
  const png = Buffer.from(
    "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAUElEQVR4nKXPMQIAwAgDQeX/nTVI8OitkLQghLg7O+3ABqzACqzACqzACqzACqzACqzACqzACqzACqzACqzACqzACqzACqzACqzACqxws00Gy5xjnXMAAAAASUVORK5CYII=",
    "base64"
  );
  return nativeImage.createFromBuffer(png);
}

function createTray() {
  tray = new Tray(trayIcon());
  tray.setToolTip("Tervixa Wiggle Pets");
  tray.setContextMenu(
    Menu.buildFromTemplate([
      { label: "Control Center", click: createDashboard },
      { label: "Chat / Quick Command (Alt+Space)", click: createChat },
      { type: "separator" },
      { label: "Show Pet Overlay", click: () => overlayWin && overlayWin.show() },
      { label: "Quit", click: () => app.quit() },
    ])
  );
  tray.on("click", createDashboard);
}

// ---------------------------------------------------------------------------
// IPC surface for renderers
// ---------------------------------------------------------------------------

ipcMain.handle("api:get", (_e, p) => apiGet(p));
ipcMain.handle("api:post", (_e, p, b) => apiPost(p, b));
ipcMain.handle("window:openDashboard", createDashboard);
ipcMain.handle("window:openChat", createChat);
ipcMain.handle("overlay:setClickThrough", (_e, on) => {
  if (overlayWin) overlayWin.setIgnoreMouseEvents(on, { forward: true });
});
ipcMain.handle("overlay:quit", () => app.quit());
ipcMain.handle("assets:dir", () => petAssetsDir());
ipcMain.handle("ai:pull", (_e, model) => {
  try {
    const p = spawn("ollama", ["pull", String(model)], { stdio: "ignore", detached: true });
    p.unref();
    return true;
  } catch {
    return false;
  }
});

app.whenReady().then(async () => {
  await ensureServer();
  createOverlay();
  createTray();
  globalShortcut.register("Alt+Space", createChat);
});

app.on("window-all-closed", () => {
  // keep running in tray unless server gone
  if (!tray) app.quit();
});

app.on("before-quit", () => {
  globalShortcut.unregisterAll();
  if (serverProc) serverProc.kill();
});
