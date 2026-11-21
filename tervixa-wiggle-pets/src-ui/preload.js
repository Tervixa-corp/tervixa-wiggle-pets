// Preload bridge — minimal, contextIsolation-safe API surface.
const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("tervixa", {
  get: (p) => ipcRenderer.invoke("api:get", p),
  post: (p, b) => ipcRenderer.invoke("api:post", p, b),
  openDashboard: () => ipcRenderer.invoke("window:openDashboard"),
  openChat: () => ipcRenderer.invoke("window:openChat"),
  setClickThrough: (on) => ipcRenderer.invoke("overlay:setClickThrough", on),
  quit: () => ipcRenderer.invoke("overlay:quit"),
  assetsDir: () => ipcRenderer.invoke("assets:dir"),
  pullModel: (m) => ipcRenderer.invoke("ai:pull", m),
});
