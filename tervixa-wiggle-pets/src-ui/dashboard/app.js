// Control Center — Agents / Wardrobe / Skills / AI tabs.
const tervixa = window.tervixa;
const view = document.getElementById("view");
const statusEl = document.getElementById("status");
let currentPet = localStorage.getItem("twp.pet") || "cloud-nimbus";

function esc(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

async function refreshStatus() {
  const h = await tervixa.get("/health");
  statusEl.textContent = h && h.ok ? `daemon v${h.data.version} · connected` : "daemon offline";
}
setInterval(refreshStatus, 4000);
refreshStatus();

document.querySelectorAll("nav a[data-tab]").forEach((a) => {
  a.onclick = () => {
    document.querySelectorAll("nav a").forEach((x) => x.classList.remove("on"));
    a.classList.add("on");
    tabs[a.dataset.tab]();
  };
});

const tabs = {
  agents,
  pets,
  skills,
  ai,
};

// --------------------------------------------------------------------------
async function agents() {
  const s = await tervixa.get("/api/sessions");
  const sessions = (s && s.ok && s.data.sessions) || [];
  view.innerHTML = `
    <h2>Watched AI Coding Agents</h2>
    <p class="sub">Wiggle Pets installs marker-scoped, idempotent hooks into each agent's own
    config so lifecycle + permission events stream to the local daemon. Hooks fail open:
    quit the pet and everything behaves exactly as stock.</p>
    <div class="row">
      <button class="primary" id="install">1-Click Install / Refresh Hooks</button>
      <button class="danger" id="uninstall">Uninstall All Hooks</button>
    </div>
    <div id="installOut" class="mono dim"></div>
    <h2>Live Sessions (${sessions.length})</h2>
    <table><tr><th>Agent</th><th>Session</th><th>State</th><th>Tool</th><th>Workspace</th><th></th></tr>
    ${sessions
      .map(
        (x) => `<tr><td>${esc(x.agent_name)}</td><td class="mono">${esc(x.session_id)}</td>
        <td>${esc(x.state)}</td><td class="mono">${esc(x.tool || "—")}</td>
        <td class="mono">${esc(x.workspace || "—")}</td>
        <td>${x.session_id.startsWith("pid-") ? `<button data-pid="${x.session_id.slice(4)}" data-agent="${esc(x.agent)}" data-sid="${esc(x.session_id)}">Interrupt</button>` : ""}</td></tr>`
      )
      .join("")}
    </table>
    <h2>Auto-Pilot Approval Policy</h2>
    <p class="sub">Deterministic rule engine — no AI. Dangerous commands are always surfaced to you.</p>
    <div class="card mono dim">auto-approve: ls, cat, grep, git status/diff/log, cargo check/test, npm test, pytest…<br/>
    block &amp; alert: rm -rf, git push --force, sudo, .ssh, .aws/credentials, …</div>`;
  document.getElementById("install").onclick = async () => {
    const r = await tervixa.post("/api/agents/install", {});
    document.getElementById("installOut").textContent = (r.data.results || [])
      .map((x) => `${x.agent}: ${x.installed ? "installed" : "FAILED — " + (x.error || "")}`)
      .join("  |  ");
  };
  document.getElementById("uninstall").onclick = async () => {
    const r = await tervixa.post("/api/agents/uninstall", {});
    document.getElementById("installOut").textContent = (r.data.results || [])
      .map((x) => `${x.agent}: ${x.removed ? "removed" : "FAILED"}`)
      .join("  |  ");
  };
  view.querySelectorAll("button[data-pid]").forEach((b) => {
    b.onclick = () =>
      tervixa.post("/api/interrupt", {
        pid: Number(b.dataset.pid),
        agent: b.dataset.agent,
        session_id: b.dataset.sid,
      });
  });
}

// --------------------------------------------------------------------------
async function pets() {
  const r = await tervixa.get("/api/pets");
  const pets = (r && r.ok && r.data) || [];
  view.innerHTML = `
    <h2>Wardrobe — ${pets.length} pets bundled</h2>
    <p class="sub">Click to wear. Pets come from VS Code Pets (MIT, see ATTRIBUTION.md) plus the
    original Tervixa Cloud Pets. Static images get procedural physics automatically.</p>
    <div class="petgrid">${pets
      .map(
        (p) => `<div class="petcard" data-id="${esc(p.id)}" data-kind="${esc(p.kind)}">
          ${p.kind === "sprite" ? `<img src="../../assets/pets/${esc(p.id)}/${esc(firstFile(p))}" />` : `<canvas width="64" height="64" data-cloud="${esc(p.id)}"></canvas>`}
          <div class="nm">${esc(p.name)}</div>
          <div class="lic">${esc(p.license || "")}</div>
        </div>`
      )
      .join("")}</div>
    <h2>GitHub Pet Explorer</h2>
    <div class="row">
      <input id="ghspec" placeholder="user/repo/pet-name (raw pet.json manifest)" style="width:380px" />
      <button class="primary" id="ghdl">Download &amp; Install</button>
    </div>
    <div id="ghout" class="mono dim"></div>`;
  view.querySelectorAll(".petcard").forEach((c) => {
    c.onclick = () => {
      currentPet = c.dataset.id;
      localStorage.setItem("twp.pet", currentPet);
      mark();
    };
  });
  view.querySelectorAll("canvas[data-cloud]").forEach((cv) => {
    drawCloudPreview(cv, pets.find((p) => p.id === cv.dataset.cloud));
  });
  document.getElementById("ghdl").onclick = async () => {
    const spec = document.getElementById("ghspec").value.trim();
    const r = await tervixa.post("/api/pets/install", { spec });
    document.getElementById("ghout").textContent = r && r.ok ? `installed: ${r.data.id}` : r.error || "download failed";
    pets();
  };
  mark();
}
function firstFile(p) {
  for (const st of Object.values(p.states)) if (st.file) return st.file;
  return "";
}
function mark() {
  document.querySelectorAll(".petcard").forEach((c) => {
    c.style.borderColor = c.dataset.id === currentPet ? "#ededed" : "";
  });
}
// draw a mini procedural cloud preview
function drawCloudPreview(cv, p) {
  const c2 = cv.getContext("2d");
  const body = p ? p.body : "#F4F6FB";
  c2.fillStyle = body;
  const puffs = [
    [20, 38, 13], [32, 26, 15], [45, 38, 13], [32, 40, 17],
  ];
  for (const [x, y, r] of puffs) {
    c2.beginPath();
    c2.arc(x, y, r, 0, Math.PI * 2);
    c2.fill();
  }
  c2.fillStyle = p ? p.eye : "#1A1D29";
  c2.beginPath(); c2.arc(27, 32, 2.2, 0, 7); c2.fill();
  c2.beginPath(); c2.arc(38, 32, 2.2, 0, 7); c2.fill();
}

// --------------------------------------------------------------------------
async function skills() {
  const hyd = await tervixa.get("/api/skills/hydration");
  const rem = await tervixa.get("/api/reminders");
  const pom = await tervixa.get("/api/pomodoro");
  const H = hyd.ok ? hyd.data : { config: {}, drunk_ml: 0, goal_ml: 2500, streak: 0 };
  const reminders = (rem.ok && rem.data.reminders) || [];
  const P = pom.ok ? pom.data : {};
  view.innerHTML = `
    <h2>Hydration Skill</h2>
    <div class="row">
      <label>every <input id="hiv" type="number" value="${H.config.interval_min || 45}" style="width:70px" /> min</label>
      <label>goal <input id="hg" type="number" value="${H.config.goal_ml || 2500}" style="width:80px" /> ml</label>
      <label>cup <input id="hc" type="number" value="${H.config.cup_ml || 250}" style="width:70px" /> ml</label>
      <label>active <input id="hs" value="${H.config.active_start || "08:00"}" style="width:70px" />–<input id="he" value="${H.config.active_end || "22:00"}" style="width:70px" /></label>
      <label class="sw"><input type="checkbox" id="hen" ${H.config.enabled ? "checked" : ""} /> enabled</label>
      <button class="primary" id="hsave">Save</button>
    </div>
    <p class="sub">Today: ${H.drunk_ml} / ${H.goal_ml} ml · streak ${H.streak} days</p>
    <h2>Pomodoro Focus</h2>
    <div class="row">
      <span class="mono">${P.active ? `${P.phase} · ${Math.round(P.remaining_secs / 60)}m left · round ${P.rounds + 1}` : "inactive"}</span>
      <button class="primary" id="pomToggle">${P.active ? "Stop" : "Start 25/5"}</button>
    </div>
    <h2>Reminders (${reminders.length})</h2>
    <div class="row">
      <input id="rtext" placeholder="remind me every monday and thursday at 6pm to back up the database" style="width:560px" />
      <button class="primary" id="radd">Add (parsed with zero AI)</button>
    </div>
    <table><tr><th>#</th><th>Kind</th><th>When</th><th>Text</th><th></th></tr>
    ${reminders
      .map(
        (r) => `<tr><td>${r.id}</td><td>${esc(r.kind)}</td>
        <td class="mono">${esc(r.daily_hhmm || r.weekdays || (r.interval_min ? `every ${r.interval_min}m` : r.at_epoch ? new Date(r.at_epoch * 1000).toLocaleString() : "—"))}</td>
        <td>${esc(r.text || r.note || "")}</td>
        <td><button data-del="${r.id}">Remove</button></td></tr>`
      )
      .join("")}
    </table>`;
  document.getElementById("hsave").onclick = async () => {
    await tervixa.post("/api/skills/hydration/config", {
      interval_min: +document.getElementById("hiv").value,
      goal_ml: +document.getElementById("hg").value,
      cup_ml: +document.getElementById("hc").value,
      active_start: document.getElementById("hs").value,
      active_end: document.getElementById("he").value,
      enabled: document.getElementById("hen").checked,
    });
    skills();
  };
  document.getElementById("pomToggle").onclick = async () => {
    await tervixa.post("/api/pomodoro", { active: !P.active });
    skills();
  };
  document.getElementById("radd").onclick = async () => {
    const text = document.getElementById("rtext").value.trim();
    if (text) await tervixa.post("/api/reminders", { text });
    skills();
  };
  view.querySelectorAll("button[data-del]").forEach((b) => {
    b.onclick = async () => {
      await tervixa.post(`/api/reminders/${b.dataset.del}/delete`, {});
      skills();
    };
  });
}

// --------------------------------------------------------------------------
async function ai() {
  const cfg = await tervixa.get("/api/ai/config");
  const cat = await tervixa.get("/api/ai/catalog");
  const C = (cfg.ok && cfg.data) || {};
  const models = (cat.ok && cat.data.models) || [];
  view.innerHTML = `
    <h2>AI is ONLY used for chat replies</h2>
    <p class="sub">Reminders, hydration, agent watching and permission approvals are deterministic
    native code — zero tokens, zero latency, fully offline. This page only configures the pet's
    conversational replies.</p>
    <div class="card">
      <label class="sw"><input type="radio" name="mode" value="off" ${C.mode === "off" ? "checked" : ""} />
        <span><b>100% No-AI Deterministic Mode</b> — chat disabled, everything else works</span></label>
      <label class="sw"><input type="radio" name="mode" value="local" ${C.mode === "local" ? "checked" : ""} />
        <span><b>Lightweight local model</b> (Ollama / LM Studio / llama.cpp)</span></label>
      <label class="sw"><input type="radio" name="mode" value="cloud" ${C.mode === "cloud" ? "checked" : ""} />
        <span><b>Cloud BYOK</b> (OpenAI-compatible endpoint or Anthropic)</span></label>
    </div>
    <div class="row">
      <input id="lbase" value="${esc(C.local_base_url || "http://localhost:11434")}" style="width:280px" />
      <input id="lmodel" value="${esc(C.local_model || "qwen2.5:1.5b")}" style="width:220px" />
      <button id="detect">Detect Local Servers</button>
    </div>
    <div id="detectOut" class="mono dim"></div>
    <h2>Lightweight Local Catalog (1-click pull via Ollama)</h2>
    <table><tr><th>Model</th><th>Params</th><th>RAM</th><th></th></tr>
    ${models
      .map(
        (m) => `<tr><td>${esc(m.name)}</td><td class="mono">${esc(m.params)}</td><td class="mono">${esc(m.ram_hint)}</td>
        <td><button data-pull="${esc(m.id)}">Pull &amp; Use</button></td></tr>`
      )
      .join("")}
    </table>
    <h2>Cloud Providers (BYOK)</h2>
    <div class="row">
      <select id="cprov">
        ${["openai", "anthropic", "openrouter", "deepseek", "glm", "kimi", "groq", "mistral"].map((p) => `<option ${C.cloud_provider === p ? "selected" : ""}>${p}</option>`).join("")}
      </select>
      <input id="cmodel" value="${esc(C.cloud_model || "gpt-4o-mini")}" style="width:220px" />
      <input id="ckenv" value="${esc(C.api_key_env || "OPENAI_API_KEY")}" style="width:200px" />
      <span class="dim mono">key is read from this env var — never stored or logged</span>
    </div>
    <div class="row"><button class="primary" id="aisave">Save Router Config</button></div>
    <div id="aiout" class="mono dim"></div>`;
  document.getElementById("detect").onclick = async () => {
    const r = await tervixa.post("/api/ai/detect", {});
    document.getElementById("detectOut").textContent = (r.data.servers || [])
      .map(([u, n]) => `${n} @ ${u}`)
      .join("  |  ") || "no local servers found — install Ollama, then `ollama pull qwen2.5:1.5b`";
  };
  view.querySelectorAll("button[data-pull]").forEach((b) => {
    b.onclick = async () => {
      document.getElementById("aiout").textContent = `pulling ${b.dataset.pull}… (runs \`ollama pull ${b.dataset.pull}\`)`;
      spawnOllamaPull(b.dataset.pull);
    };
  });
  document.getElementById("aisave").onclick = async () => {
    const mode = document.querySelector('input[name=mode]:checked')?.value || "local";
    await tervixa.post("/api/ai/config", {
      mode,
      local_base_url: document.getElementById("lbase").value,
      local_model: document.getElementById("lmodel").value,
      cloud_provider: document.getElementById("cprov").value,
      cloud_model: document.getElementById("cmodel").value,
      api_key_env: document.getElementById("ckenv").value,
    });
    document.getElementById("aiout").textContent = "router config saved";
  };
}

function spawnOllamaPull(model) {
  // best-effort; the user can always run it manually
  if (tervixa.pullModel) tervixa.pullModel(model);
}

tabs.agents();
