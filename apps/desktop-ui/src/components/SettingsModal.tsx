import { useEffect, useState } from "react";
import * as api from "../api";

const SECTIONS = [
  "providers",
  "models",
  "permissions",
  "hooks",
  "mcp",
  "skills",
  "memory",
  "browser",
  "swarm",
  "about",
] as const;

type Section = (typeof SECTIONS)[number];

interface Props {
  status: api.EngineStatus | null;
  onClose: () => void;
  onSaved: () => void;
}

export default function SettingsModal({ status, onClose, onSaved }: Props) {
  const [section, setSection] = useState<Section>("providers");
  const [config, setConfig] = useState<Record<string, unknown>>({});
  const [mcpStates, setMcpStates] = useState<api.McpStateView[]>([]);
  const [error, setError] = useState("");

  useEffect(() => {
    api.settings().then(setConfig).catch((e) => setError(String(e)));
  }, []);

  const save = async () => {
    try {
      await api.saveSettings(config);
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    }
  };

  const patchProvider = (idx: number, patch: Partial<api.ProviderView>) => {
    const providers = [...((config.providers as api.ProviderView[]) ?? [])];
    providers[idx] = { ...providers[idx], ...patch };
    setConfig({ ...config, providers });
  };

  const set = (key: string, value: unknown) => setConfig({ ...config, [key]: value });

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="nav">
          {SECTIONS.map((s) => (
            <button key={s} className={section === s ? "active" : ""} onClick={() => setSection(s)}>
              {s}
            </button>
          ))}
        </div>
        <div className="content">
          {error && <pre style={{ color: "var(--danger)" }}>{error}</pre>}

          {section === "providers" &&
            ((config.providers as api.ProviderView[]) ?? []).map((p, i) => (
              <div key={p.id} style={{ marginBottom: 14 }}>
                <label>{p.label || p.id}</label>
                <div style={{ display: "flex", gap: 8 }}>
                  <select
                    value={p.enabled ? "on" : "off"}
                    onChange={(e) => patchProvider(i, { enabled: e.target.value === "on" })}
                  >
                    <option value="on">enabled</option>
                    <option value="off">disabled</option>
                  </select>
                  <input
                    style={{ flex: 1 }}
                    value={p.base_url}
                    onChange={(e) => patchProvider(i, { base_url: e.target.value })}
                  />
                </div>
              </div>
            ))}

          {section === "models" && (
            <>
              <label>Default provider</label>
              <input
                value={(config.default_provider as string) ?? ""}
                onChange={(e) => set("default_provider", e.target.value)}
              />
              <label>Default model</label>
              <input
                value={(config.default_model as string) ?? ""}
                onChange={(e) => set("default_model", e.target.value)}
              />
            </>
          )}

          {section === "permissions" && (
            <>
              <label>Sandbox mode</label>
              <select
                value={
                  ((config.permissions as Record<string, unknown>)?.mode as string) ?? "suggest"
                }
                onChange={(e) =>
                  set("permissions", {
                    ...(config.permissions as Record<string, unknown>),
                    mode: e.target.value,
                  })
                }
              >
                <option value="suggest">suggest (approve every edit)</option>
                <option value="auto-edit">auto-edit (edits auto, commands ask)</option>
                <option value="full-auto">full-auto</option>
              </select>
            </>
          )}

          {section === "hooks" && (
            <label>Lifecycle hooks are configured in .wiggle/config.toml
              <textarea
                rows={8}
                value={JSON.stringify(config.hooks ?? [], null, 2)}
                onChange={(e) => {
                  try {
                    set("hooks", JSON.parse(e.target.value));
                  } catch {
                    /* keep editing */
                  }
                }}
              />
            </label>
          )}

          {section === "mcp" && (
            <>
              <button
                className="mem-row button"
                style={{ padding: "4px 10px", marginBottom: 10 }}
                onClick={async () => setMcpStates(await api.connectMcpAll().catch(() => []))}
              >
                connect all
              </button>
              {(mcpStates.length === 0
                ? [{ name: "—", transport: "—", connected: false, error: null, tools: 0, resources: 0, prompts: 0 }]
                : mcpStates
              ).map((s, i) => (
                <div className="mcp-row" key={i}>
                  <span className={s.connected ? "ok" : "bad"}>
                    {s.connected ? "●" : "○"} {s.name} [{s.transport}] tools={s.tools}
                  </span>
                </div>
              ))}
            </>
          )}

          {section === "skills" && (
            <>
              <pre>{status?.skills ?? 0} skills loaded (bundled + user + project)</pre>
              <label>Import from Claude Code (~/.claude/skills)</label>
              <button
                style={{ marginTop: 6 }}
                onClick={async () => alert(`imported ${await api.importSkillsClaude()} skills`)}
              >
                import local
              </button>
              <label>Import from GitHub (owner/name of a SKILL.md repo)</label>
              <input
                placeholder="anthropics/skills"
                onKeyDown={async (e) => {
                  if (e.key === "Enter") {
                    const n = await api
                      .importSkillsGithub((e.target as HTMLInputElement).value)
                      .catch((err) => {
                        alert(String(err));
                        return 0;
                      });
                    alert(`imported ${n} skills`);
                  }
                }}
              />
            </>
          )}

          {section === "memory" && (
            <label>
              <input
                type="checkbox"
                checked={(config.memory_enabled as boolean) ?? true}
                onChange={(e) => set("memory_enabled", e.target.checked)}
              />{" "}
              enable the memory vault (~/.wiggle/memory/wiggle.db)
            </label>
          )}

          {section === "browser" && (
            <pre>
              The browser pane drives the engine's bundled CDP controller
              (Chrome / Edge). Tools: browser_navigate, browser_actuate,
              browser_inspect.
            </pre>
          )}

          {section === "swarm" && (
            <pre>
              Parallel multi-agent swarms coordinate through in-memory file
              leases — never git branches. The compiler routes prompts with
              parallel intent ("and then", "in parallel", "meanwhile") to a
              coder + explorer pair automatically.
            </pre>
          )}

          {section === "about" && (
            <pre>
              {status?.product ?? "Tervixa Wiggle"} v{status?.version ?? "?"}
              {"\n"}MIT licensed. Part of the Tervixa Forge toolchain.
            </pre>
          )}
        </div>
        <div className="footer">
          <button onClick={onClose}>cancel</button>
          <button className="primary" onClick={save}>
            save
          </button>
        </div>
      </div>
    </div>
  );
}
