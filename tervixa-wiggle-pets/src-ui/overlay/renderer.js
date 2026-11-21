// TERVIXA WIGGLE PETS — overlay renderer.
// 60 fps pet loop: sprite/gif rendering, procedural clouds & bot, walk/run
// physics, bezel climbing, cursor eye-tracking, drag wiggle, ball fetch,
// agent-state synchronization, and the floating bubble cards.

const tervixa = window.tervixa;

const canvas = document.getElementById("scene");
const ctx = canvas.getContext("2d");
const spriteEl = document.getElementById("sprite");
const bubblesEl = document.getElementById("bubbles");
const hudEl = document.getElementById("hud");

let WA = { x: 0, y: 0, width: window.innerWidth, height: window.innerHeight };
let DPR = Math.min(window.devicePixelRatio || 1, 2);

function resize() {
  WA = { x: 0, y: 0, width: window.innerWidth, height: window.innerHeight };
  DPR = Math.min(window.devicePixelRatio || 1, 2);
  canvas.width = WA.width * DPR;
  canvas.height = WA.height * DPR;
  ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
}
window.addEventListener("resize", resize);
resize();

// ---------------------------------------------------------------------------
// Pet model
// ---------------------------------------------------------------------------

const pet = {
  manifest: null,
  state: "idle",
  stateUntil: 0,
  facing: 1,
  x: 160,
  y: WA.height - 60, // top-left of sprite
  vx: 0,
  vy: 0,
  scale: 2,
  groundOffset: 6,
  dragging: false,
  dragDX: 0,
  dragDY: 0,
  wiggle: 0,
  eye: { x: 0, y: 0 }, // cursor offset for pupils
  onBezel: null, // "left" | "right" | "top" | null
  frame: 0,
  imgs: {}, // state -> HTMLImageElement
  ball: null,
  sessions: [],
  lastEventSeq: 0,
  mouse: { x: WA.width / 2, y: WA.height / 2, idleMs: 0 },
};

async function loadPet(id) {
  const res = await tervixa.get("/api/pets");
  const pets = (res && res.ok && res.data) || [];
  const m = pets.find((p) => p.id === id) || pets.find((p) => p.id === "cloud-nimbus") || pets[0];
  if (!m) return;
  pet.manifest = m;
  pet.scale = m.scale || 2;
  pet.groundOffset = m.ground_offset || 6;
  pet.imgs = {};
  if (m.kind === "sprite") {
    for (const [state, st] of Object.entries(m.states)) {
      if (st.file) {
        const img = new Image();
        img.src = `../../assets/pets/${m.id}/${st.file}`;
        pet.imgs[state] = img;
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Agent state driver
// ---------------------------------------------------------------------------

const STATE_PRIORITY = { notification: 5, error: 4, working: 3, thinking: 2, attention: 1 };

function derivePetState() {
  let top = null;
  for (const s of pet.sessions) {
    if (!STATE_PRIORITY[s.state]) continue;
    if (!top || STATE_PRIORITY[s.state] > STATE_PRIORITY[top.state]) top = s;
  }
  if (top) return top.state;
  if (pet.mouse.idleMs > 60_000) return "sleeping";
  return "idle";
}

// ---------------------------------------------------------------------------
// Physics / behavior
// ---------------------------------------------------------------------------

const SPEED = 34; // px/s walking

function groundY() {
  return WA.height - pet.height() - pet.groundOffset;
}

pet.height = () => (pet.manifest ? (pet.manifest.size ? pet.manifest.size[1] : 40) * pet.scale : 80);
pet.width = () => (pet.manifest ? (pet.manifest.size ? pet.manifest.size[0] : 48) * pet.scale : 96);

let nextDecision = 0;

function tick(dt, now) {
  // eye tracking
  const cx = pet.x + pet.width() / 2;
  const cy = pet.y + pet.height() / 2;
  const dx = pet.mouse.x - cx;
  const dy = pet.mouse.y - cy;
  const d = Math.hypot(dx, dy) || 1;
  pet.eye.x += ((dx / d) * Math.min(2.4, d / 40) - pet.eye.x) * 0.08;
  pet.eye.y += ((dy / d) * Math.min(1.8, d / 60) - pet.eye.y) * 0.08;

  if (pet.dragging) {
    pet.wiggle = Math.sin(now / 40) * 0.14;
    pet.ball = null;
    return;
  }
  pet.wiggle *= 0.9;

  // state machine decisions every ~2.5s
  if (now > nextDecision) {
    nextDecision = now + 2200 + Math.random() * 2600;
    const st = derivePetState();
    pet.state = st;
    if (st === "sleeping" || st === "notification" || st === "error") {
      pet.vx = 0;
    } else if (st === "working") {
      pet.vx = pet.facing * SPEED * 1.6;
    } else if (st === "idle" || st === "attention") {
      const r = Math.random();
      if (pet.onBezel) {
        pet.vx = 0;
        if (r < 0.4) pet.onBezel = null; // hop off the bezel
      } else if (r < 0.45) {
        pet.vx = 0; // rest
      } else if (r < 0.5 && pet.y < groundY() - 40) {
        pet.vx = 0; // stay on wall
      } else {
        pet.facing = Math.random() < 0.5 ? -1 : 1;
        pet.vx = pet.facing * SPEED;
      }
    }
  }

  // ball fetch physics
  if (pet.ball) {
    const b = pet.ball;
    b.vy += 900 * dt;
    b.x += b.vx * dt;
    b.y += b.vy * dt;
    const floor = groundY() + pet.height() - 8;
    if (b.y >= floor) {
      b.y = floor;
      b.vy *= -0.45;
      b.vx *= 0.8;
      if (Math.abs(b.vy) < 30) {
        pet.ball = null;
        pet.state = "attention";
      }
    }
    // chase
    const bx = b.x - pet.x;
    pet.facing = bx < 0 ? -1 : 1;
    pet.vx = Math.sign(bx) * SPEED * 2.4;
    if (Math.abs(bx) < 24 && Math.abs(b.y - pet.y) < 48) pet.ball = null;
  }

  // bezel climbing
  if (pet.onBezel) {
    if (pet.onBezel === "left") {
      pet.x = WA.x + 4;
      pet.y += (pet.mouse.y - pet.y - pet.height() / 2) * 0.02;
    } else if (pet.onBezel === "right") {
      pet.x = WA.x + WA.width - pet.width() - 4;
      pet.y += (pet.mouse.y - pet.y - pet.height() / 2) * 0.02;
    } else if (pet.onBezel === "top") {
      pet.y = WA.y + 2;
      pet.x += (pet.mouse.x - pet.x - pet.width() / 2) * 0.02;
    }
    pet.y = Math.max(WA.y + 2, Math.min(groundY(), pet.y));
    return;
  }

  // integrate
  pet.x += pet.vx * dt;
  pet.y = Math.min(groundY(), pet.y + 400 * dt); // gravity settles to ground

  // clamp + bezel stick on walk
  if (pet.x < WA.x + 4) {
    pet.x = WA.x + 4;
    if (pet.vx < 0) maybeClimb("left");
  }
  if (pet.x + pet.width() > WA.x + WA.width - 4) {
    pet.x = WA.x + WA.width - pet.width() - 4;
    if (pet.vx > 0) maybeClimb("right");
  }
}

function maybeClimb(side) {
  // Kimi-style: sometimes climb the screen bezel and hang there
  if (Math.random() < 0.35) {
    pet.onBezel = side;
    pet.vx = 0;
    nextDecision = performance.now() + 4000 + Math.random() * 5000;
  } else {
    pet.facing *= -1;
    pet.vx = pet.facing * SPEED;
  }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

function render(now) {
  ctx.clearRect(0, 0, WA.width, WA.height);
  const m = pet.manifest;
  if (!m) return;

  // ball
  if (pet.ball) {
    ctx.fillStyle = "#EDEDED";
    ctx.beginPath();
    ctx.arc(pet.ball.x, pet.ball.y, 5 * pet.scale * 0.6, 0, Math.PI * 2);
    ctx.fill();
  }

  if (m.kind === "procedural") {
    spriteEl.style.display = "none";
    drawProcedural(now);
  } else {
    ctx.clearRect(pet.prevX ?? 0, pet.prevY ?? 0, pet.width() + 20, pet.height() + 20);
    const img = pet.imgs[pet.state] || pet.imgs["idle"];
    if (img && img.complete && img.naturalWidth) {
      spriteEl.style.display = "block";
      spriteEl.src = img.src; // gif keeps its own timeline
      const flip = pet.facing === -1;
      spriteEl.style.left = `${pet.x}px`;
      spriteEl.style.top = `${pet.y}px`;
      spriteEl.style.width = `${pet.width()}px`;
      spriteEl.style.height = `${pet.height()}px`;
      spriteEl.style.transform = flip ? "scaleX(-1)" : "none";
    }
  }
  pet.prevX = pet.x;
  pet.prevY = pet.y;
}

function drawProcedural(now) {
  const st = pet.procPose ? pet.procPose : pet.state;
  const t = now / 1000;
  const w = pet.width();
  const h = pet.height();
  const bob = Math.sin(t * 2.1) * 3;
  ctx.save();
  ctx.translate(pet.x + w / 2, pet.y + h / 2 + (pet.onBezel ? 0 : bob));
  ctx.scale(pet.facing, 1);
  ctx.rotate(pet.wiggle);

  if (pet.manifest.style === "bot") drawBot(st, t, w, h);
  else drawCloud(st, t, w, h);
  ctx.restore();
}

function drawCloud(st, t, w, h) {
  const c = pet.manifest;
  const body = c.body || "#F4F6FB";
  const shade = c.shade || "#C9D2E8";
  const eyeC = c.eye || "#1A1D29";
  const accent = c.accent || "#7FB2FF";

  // stormy lightning
  if (st === "notification") {
    if (Math.sin(t * 9) > 0.4) {
      ctx.fillStyle = accent;
      ctx.beginPath();
      ctx.moveTo(-6, h * 0.1);
      ctx.lineTo(6, -h * 0.05);
      ctx.lineTo(0, -h * 0.02);
      ctx.lineTo(8, h * 0.12);
      ctx.lineTo(-2, h * 0.06);
      ctx.lineTo(2, h * 0.02);
      ctx.closePath();
      ctx.fill();
    }
  }

  // cloud body: composed puffs
  ctx.fillStyle = body;
  const puffs = [
    [-w * 0.26, h * 0.06, w * 0.24],
    [0, -h * 0.16, w * 0.3],
    [w * 0.26, h * 0.06, w * 0.24],
    [0, h * 0.08, w * 0.34],
  ];
  for (const [px, py, r] of puffs) {
    ctx.beginPath();
    ctx.arc(px, py, r, 0, Math.PI * 2);
    ctx.fill();
  }
  // subtle under-shade
  ctx.fillStyle = shade;
  ctx.globalAlpha = 0.5;
  ctx.beginPath();
  ctx.ellipse(0, h * 0.24, w * 0.32, h * 0.1, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.globalAlpha = 1;

  // working = code rain
  if (st === "working") {
    ctx.fillStyle = accent;
    for (let i = 0; i < 4; i++) {
      const rx = -w * 0.3 + ((i * 97) % (w * 0.6));
      const ry = h * 0.3 + ((t * 60 + i * 23) % (h * 0.5));
      ctx.fillRect(rx, ry, 3, 6);
    }
  }
  if (st === "drinking_water") {
    ctx.fillStyle = accent;
    for (let i = 0; i < 3; i++) {
      const rx = -w * 0.2 + i * w * 0.2;
      const ry = h * 0.3 + ((t * 80 + i * 30) % (h * 0.5));
      ctx.beginPath();
      ctx.arc(rx, ry, 2.2, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  // face
  const eyesClosed = st === "sleeping";
  ctx.fillStyle = eyeC;
  if (eyesClosed) {
    ctx.fillRect(-w * 0.12, -h * 0.04, w * 0.07, 2);
    ctx.fillRect(w * 0.05, -h * 0.04, w * 0.07, 2);
  } else {
    const ex = pet.eye.x, ey = pet.eye.y;
    for (const s of [-1, 1]) {
      ctx.beginPath();
      ctx.arc(s * w * 0.09, -h * 0.02, 4.4, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = body;
      ctx.beginPath();
      ctx.arc(s * w * 0.09 + ex, -h * 0.02 + ey, 1.9, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = eyeC;
    }
  }
  // mouth per state
  ctx.strokeStyle = eyeC;
  ctx.lineWidth = 1.6;
  ctx.beginPath();
  if (st === "error") {
    ctx.moveTo(-5, h * 0.12); ctx.lineTo(5, h * 0.16);
    ctx.moveTo(5, h * 0.12); ctx.lineTo(-5, h * 0.16);
    ctx.stroke();
  } else if (st === "attention" || st === "drinking_water") {
    ctx.arc(0, h * 0.08, 5, 0.15 * Math.PI, 0.85 * Math.PI);
    ctx.stroke();
  } else if (st === "sleeping") {
    ctx.arc(0, h * 0.1, 4, 0.1 * Math.PI, 0.9 * Math.PI);
    ctx.stroke();
  }

  // zzz / thought marks
  ctx.fillStyle = shade;
  ctx.font = `600 ${Math.round(w * 0.14)}px JetBrains Mono, monospace`;
  if (st === "sleeping") {
    for (let i = 0; i < 3; i++) {
      const a = (t * 0.8 + i * 0.5) % 1.6;
      ctx.globalAlpha = 1 - a / 1.6;
      ctx.fillText("z", w * 0.28 + a * 10, -h * 0.3 - a * 16);
    }
    ctx.globalAlpha = 1;
  }
  if (st === "thinking") {
    ctx.globalAlpha = 0.6 + 0.4 * Math.sin(t * 3);
    ctx.fillText("…", w * 0.3, -h * 0.34);
    ctx.globalAlpha = 1;
  }
}

function drawBot(st, t, w, h) {
  const c = pet.manifest;
  const body = c.body || "#1E63B4";
  const dark = c.shade || "#0F3B78";
  const eyeC = c.eye || "#FFFFFF";
  const accent = c.accent || "#F5C542";
  const u = w / 12; // pixel unit

  // antenna
  ctx.fillStyle = dark;
  ctx.fillRect(-u * 0.4, -h / 2 - u * 2, u * 0.8, u * 2.2);
  ctx.fillStyle = st === "notification" ? accent : body;
  ctx.beginPath();
  ctx.arc(0, -h / 2 - u * 2.4, u * 1.1, 0, Math.PI * 2);
  ctx.fill();
  if (st === "working" || st === "thinking") {
    ctx.globalAlpha = 0.4 + 0.6 * Math.abs(Math.sin(t * 6));
    ctx.fillStyle = accent;
    ctx.beginPath();
    ctx.arc(0, -h / 2 - u * 2.4, u * 1.6, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalAlpha = 1;
  }

  // head
  ctx.fillStyle = body;
  ctx.fillRect(-w * 0.42, -h * 0.42, w * 0.84, h * 0.5);
  // face screen
  ctx.fillStyle = "#0A0A0A";
  ctx.fillRect(-w * 0.32, -h * 0.32, w * 0.64, h * 0.3);
  // eyes track cursor
  const ex = pet.eye.x * 0.7, ey = pet.eye.y * 0.7;
  ctx.fillStyle = eyeC;
  if (st === "error") {
    drawX(-u * 1.8, -h * 0.2, u * 0.9);
    drawX(u * 1.8, -h * 0.2, u * 0.9);
  } else if (st === "sleeping") {
    ctx.fillRect(-u * 2.4, -h * 0.18, u * 1.2, u * 0.4);
    ctx.fillRect(u * 1.2, -h * 0.18, u * 1.2, u * 0.4);
  } else {
    for (const s of [-1, 1]) {
      ctx.beginPath();
      ctx.arc(s * u * 1.8 + ex, -h * 0.2 + ey, u * 0.8, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  // body
  ctx.fillStyle = dark;
  ctx.fillRect(-w * 0.34, h * 0.1, w * 0.68, h * 0.34);
  // chest pixel (status LED)
  ctx.fillStyle =
    st === "error" ? "#D4544A" :
    st === "notification" ? "#58B06B" :
    st === "working" ? accent : "#8A8A8A";
  ctx.fillRect(-u * 0.7, h * 0.2, u * 1.4, u * 1.4);

  // arms wave on attention
  ctx.fillStyle = body;
  const armY = h * 0.12;
  if (st === "attention") {
    ctx.save();
    ctx.translate(w * 0.34, armY);
    ctx.rotate(Math.sin(t * 10) * 0.7);
    ctx.fillRect(0, 0, u * 1.2, u * 3);
    ctx.restore();
    ctx.fillRect(-w * 0.34 - u * 1.2, armY, u * 1.2, u * 3);
  } else {
    ctx.fillRect(-w * 0.34 - u * 1.2, armY, u * 1.2, u * 3);
    ctx.fillRect(w * 0.34, armY, u * 1.2, u * 3);
  }
  // typing hands when working
  if (st === "working") {
    ctx.fillStyle = accent;
    const yy = h * 0.44 + Math.abs(Math.sin(t * 12)) * u;
    ctx.fillRect(-w * 0.2, yy, u * 1.4, u * 0.8);
    ctx.fillRect(w * 0.2 - u * 1.4, h * 0.44 + Math.abs(Math.sin(t * 12 + 1)) * u, u * 1.4, u * 0.8);
  }
}

function drawX(x, y, s) {
  ctx.strokeStyle = "#D4544A";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(x - s, y - s); ctx.lineTo(x + s, y + s);
  ctx.moveTo(x + s, y - s); ctx.lineTo(x - s, y + s);
  ctx.stroke();
}

// ---------------------------------------------------------------------------
// Bubbles
// ---------------------------------------------------------------------------

const activeBubbles = new Map(); // key -> element

function bubble(key, html, ttlMs) {
  let el = activeBubbles.get(key);
  if (!el) {
    el = document.createElement("div");
    el.className = "bubble";
    bubblesEl.appendChild(el);
    activeBubbles.set(key, el);
  }
  el.innerHTML = html;
  el.style.left = `${Math.min(WA.width - 380, Math.max(8, pet.x + pet.width() / 2 - 180))}px`;
  el.style.top = `${Math.max(8, pet.y - el.offsetHeight - 16)}px`;
  if (ttlMs) {
    clearTimeout(el._ttl);
    el._ttl = setTimeout(() => el.remove(), ttlMs);
  }
  return el;
}

function renderPermissionBubble(ask) {
  const key = `perm-${ask.id}`;
  const agent = (ask.agent || "agent").toUpperCase();
  const html = `
    <div class="title"><span>${esc(agent)}${ask.workspace ? ` · ${esc(shortRepo(ask.workspace))}` : ""}</span><span>${esc(ask.tool || "")}</span></div>
    <div>Permission needed:</div>
    <div class="subject">${esc(ask.subject || "(no detail)")}</div>
    <div class="row">
      <button class="primary" data-act="allow">Approve (Enter)</button>
      <button data-act="allow_always">Always Allow</button>
      <button class="danger" data-act="deny">Deny (Esc)</button>
      <input type="text" placeholder="Reply to the agent…" data-role="reply" />
      <button data-act="reply">Send</button>
    </div>`;
  const el = bubble(key, html);
  if (!el._bound) {
    el._bound = true;
    el.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.target.matches("input")) answer(ask.id, "allow");
      if (e.key === "Escape") answer(ask.id, "deny");
    });
    el.addEventListener("click", (e) => {
      const act = e.target.getAttribute && e.target.getAttribute("data-act");
      if (!act) return;
      if (act === "reply") {
        const input = el.querySelector("[data-role=reply]");
        if (input && input.value.trim()) answer(ask.id, "reply", input.value.trim());
      } else {
        answer(ask.id, act);
      }
    });
  }
}

async function answer(id, decision, text) {
  await tervixa.post("/api/permission/answer", { id, decision: text ? "reply" : decision, ...(text ? {} : {}), ...(decision === "reply" ? {} : {}), reply: text || undefined });
  const el = activeBubbles.get(`perm-${id}`);
  if (el) el.remove();
}

function renderHydrationBubble(h) {
  const pct = Math.min(100, Math.round((h.drunk_ml / Math.max(1, h.goal_ml)) * 100));
  const html = `
    <div class="title"><span>Hydration</span><span>${h.cups} cups · streak ${h.streak}d</span></div>
    <div>Time to drink water! (${h.cup_ml} ml)</div>
    <div class="meter"><i style="width:${pct}%"></i></div>
    <div class="row">
      <button class="primary" data-h="log">Drank It! (+${h.cup_ml}ml)</button>
      <button data-h="snooze">Snooze 15m</button>
    </div>`;
  const el = bubble("hydration", html, 90_000);
  if (!el._bound) {
    el._bound = true;
    el.addEventListener("click", async (e) => {
      const act = e.target.getAttribute("data-h");
      if (act === "log") {
        await tervixa.post("/api/skills/hydration/log", { ml: h.cup_ml });
        el.remove();
      } else if (act === "snooze") {
        el.remove();
      }
    });
  }
}

function renderReminderBubble(r) {
  const html = `
    <div class="title"><span>Reminder</span><span>${esc(r.kind)}</span></div>
    <div>${esc(r.text)}</div>
    <div class="row"><button class="primary" data-r="done">Done</button><button data-r="snooze">Snooze 10m</button></div>`;
  const el = bubble(`rem-${r.id}`, html, 120_000);
  if (!el._bound) {
    el._bound = true;
    el.addEventListener("click", async (e) => {
      const act = e.target.getAttribute("data-r");
      if (act === "done") {
        el.remove();
      } else if (act === "snooze") {
        await tervixa.post("/api/reminders", { text: `remind me in 10 minutes to ${r.text}` });
        el.remove();
      }
    });
  }
}

function esc(s) {
  return String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}
function shortRepo(p) {
  const parts = String(p).replace(/\\/g, "/").split("/").filter(Boolean);
  return parts[parts.length - 1] || p;
}

// ---------------------------------------------------------------------------
// Polling (deterministic: plain HTTP every interval — no websockets needed)
// ---------------------------------------------------------------------------

let lastSeq = 0;

async function pollEvents() {
  const res = await tervixa.get(`/api/events?since=${lastSeq}`);
  if (!res || !res.ok) return;
  for (const ev of res.data.events || []) {
    lastSeq = Math.max(lastSeq, ev.seq);
    if (ev.kind === "permission" && ev.payload.resolved === "pending") {
      renderPermissionBubble(ev.payload.ask);
      pet.state = "notification";
      nextDecision = performance.now() + 6000;
    } else if (ev.kind === "hydration") {
      renderHydrationBubble(ev.payload);
      pet.state = "drinking_water";
      nextDecision = performance.now() + 8000;
    } else if (ev.kind === "reminder") {
      renderReminderBubble(ev.payload);
      pet.state = "notification";
      nextDecision = performance.now() + 6000;
    } else if (ev.kind === "quick") {
      toastReply(ev.payload.reply);
    } else if (ev.kind === "state") {
      // celebration on completion
      if (ev.payload.state === "attention") {
        pet.state = "attention";
        nextDecision = performance.now() + 5000;
      }
    }
  }
}

function toastReply(text) {
  const html = `<div class="title"><span>Pet</span></div><div>${esc(text)}</div>`;
  bubble("reply", html, 10_000);
}

async function pollSessions() {
  const res = await tervixa.get("/api/sessions");
  if (res && res.ok) {
    pet.sessions = res.data.sessions || [];
    const lines = pet.sessions
      .slice(0, 6)
      .map(
        (s) =>
          `<div><span class="dot ${s.state}"></span><b>${esc(s.agent_name)}</b> ${esc(s.state)}${s.tool ? ` · ${esc(s.tool)}` : ""}${
            s.session_id.startsWith("pid-")
              ? ` <button style="pointer-events:auto" data-pid="${s.session_id.slice(4)}" data-agent="${esc(s.agent)}" data-sid="${esc(s.session_id)}">Interrupt</button>`
              : ""
          }</div>`
      )
      .join("");
    hudEl.innerHTML = lines;
  }
}

hudEl.addEventListener("click", async (e) => {
  const pid = e.target.getAttribute("data-pid");
  if (!pid) return;
  await tervixa.post("/api/interrupt", {
    pid: Number(pid),
    agent: e.target.getAttribute("data-agent"),
    session_id: e.target.getAttribute("data-sid"),
  });
});

// ---------------------------------------------------------------------------
// Input: drag, click, double-click (ball), mouse tracking
// ---------------------------------------------------------------------------

function overPet(x, y) {
  return x >= pet.x - 6 && x <= pet.x + pet.width() + 6 && y >= pet.y - 6 && y <= pet.y + pet.height() + 6;
}

window.addEventListener("mousemove", (e) => {
  pet.mouse.x = e.clientX;
  pet.mouse.y = e.clientY;
  pet.mouse.idleMs = 0;
  if (pet.dragging) {
    pet.x = e.clientX - pet.dragDX;
    pet.y = e.clientY - pet.dragDY;
    pet.onBezel = null;
  }
  // click-through forwarding when not over the pet
  tervixa.setClickThrough(!overPet(e.clientX, e.clientY) && bubblesEl.childElementCount === 0);
});
window.addEventListener("mouseout", () => (pet.mouse.idleMs = 0));
setInterval(() => (pet.mouse.idleMs += 250), 250);

window.addEventListener("mousedown", (e) => {
  if (overPet(e.clientX, e.clientY)) {
    pet.dragging = true;
    pet.dragDX = e.clientX - pet.x;
    pet.dragDY = e.clientY - pet.y;
    pet.vx = 0;
  }
});
window.addEventListener("mouseup", (e) => {
  if (pet.dragging) {
    pet.dragging = false;
    if (pet.x < 40 || pet.x > WA.width - 60) maybeClimb(pet.x < 40 ? "left" : "right");
    pet.onBezel = null;
    pet.y = groundY();
  }
});
window.addEventListener("dblclick", (e) => {
  if (overPet(e.clientX, e.clientY)) {
    pet.ball = { x: e.clientX, y: e.clientY, vx: (Math.random() - 0.5) * 300, vy: -260 };
    pet.state = "working";
  }
});

// context menu → Control Center
window.addEventListener("contextmenu", (e) => {
  if (overPet(e.clientX, e.clientY)) {
    e.preventDefault();
    tervixa.openDashboard();
  }
});

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

let prev = performance.now();
function frame(now) {
  const dt = Math.min(0.05, (now - prev) / 1000);
  prev = now;
  tick(dt, now);
  render(now);
  requestAnimationFrame(frame);
}

async function boot() {
  await loadPet("cloud-nimbus");
  requestAnimationFrame(frame);
  setInterval(pollEvents, 900);
  setInterval(pollSessions, 2500);
  pollEvents();
  pollSessions();
}
boot();
