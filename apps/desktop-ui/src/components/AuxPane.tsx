import { useCallback, useEffect, useState } from "react";
import * as api from "../api";
import type { StreamItem, } from "../App";
import type { EngineStatus } from "../api";

export type AuxTab = "output" | "memory" | "browser" | "raw";

interface Props {
  tab: AuxTab;
  onTab: (t: AuxTab) => void;
  items: StreamItem[];
  status: EngineStatus | null;
}

const TABS: AuxTab[] = ["output", "memory", "browser", "raw"];

export default function AuxPane({ tab, onTab, items, status }: Props) {
  return (
    <div className="aux">
      <div className="tabs">
        {TABS.map((t) => (
          <button key={t} className={tab === t ? "active" : ""} onClick={() => onTab(t)}>
            {t}
          </button>
        ))}
      </div>
      <div className="pane">
        {tab === "output" && (
          <pre>
            {items
              .filter((i) => i.kind === "tool")
              .map((i) => `${i.ok ? "✓" : "✗"} ${i.title}\n${i.text}\n`)
              .join("\n") || "(tool output appears here)"}
          </pre>
        )}
        {tab === "memory" && <MemoryPane />}
        {tab === "browser" && <BrowserPane status={status} />}
        {tab === "raw" && <RawPane />}
      </div>
    </div>
  );
}

function MemoryPane() {
  const [entries, setEntries] = useState<api.MemoryEntryView[]>([]);
  const [draft, setDraft] = useState("");

  const reload = useCallback(async () => {
    try {
      setEntries(await api.memoryList(100));
    } catch {
      setEntries([]);
    }
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  return (
    <div>
      <div style={{ display: "flex", gap: 6, marginBottom: 10 }}>
        <input
          style={{ flex: 1 }}
          placeholder="Add a fact the agent should always remember…"
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={async (e) => {
            if (e.key === "Enter" && draft.trim()) {
              await api.memoryAdd(draft.trim()).catch(() => undefined);
              setDraft("");
              reload();
            }
          }}
        />
      </div>
      {entries.map((e) => (
        <div className="mem-row" key={e.id}>
          <span className="kind">{e.kind}</span>
          <span style={{ flex: 1 }}>{e.content}</span>
          <button
            onClick={async () => {
              await api.memoryPin(e.id, !e.pinned).catch(() => undefined);
              reload();
            }}
          >
            {e.pinned ? "unpin" : "pin"}
          </button>
          <button
            onClick={async () => {
              await api.memoryDelete(e.id).catch(() => undefined);
              reload();
            }}
          >
            del
          </button>
        </div>
      ))}
      {entries.length === 0 && <pre>(vault empty)</pre>}
    </div>
  );
}

function BrowserPane({ status }: { status: EngineStatus | null }) {
  const [url, setUrl] = useState("http://localhost:1420");
  const [log, setLog] = useState("");
  const [loading, setLoading] = useState(false);

  const run = async () => {
    setLoading(true);
    setLog("");
    try {
      // The engine exposes browser tools through the normal tool loop.
      const reply = await api.sendPrompt(
        "browser-main",
        `Use the browser tools to open ${url} and report what you see on the page.`,
      );
      setLog(reply);
    } catch (e) {
      setLog(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <div style={{ display: "flex", gap: 6, marginBottom: 10 }}>
        <input
          style={{ flex: 1 }}
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="http://…"
        />
        <button className="mem-row button" style={{ padding: "4px 10px" }} disabled={loading} onClick={run}>
          {loading ? "…" : "inspect"}
        </button>
      </div>
      <pre>{log || `(workspace: ${status?.workspace ?? "…"})`}</pre>
    </div>
  );
}

function RawPane() {
  const [states, setStates] = useState<api.McpStateView[]>([]);

  useEffect(() => {
    api
      .connectMcpAll()
      .then(setStates)
      .catch(() => setStates([]));
  }, []);

  return (
    <pre>
      {states.length === 0
        ? "(no MCP servers configured)"
        : states
            .map(
              (s) =>
                `${s.connected ? "●" : "○"} ${s.name} [${s.transport}]\n  tools=${s.tools} resources=${s.resources} prompts=${s.prompts}${s.error ? `\n  error: ${s.error}` : ""}`,
            )
            .join("\n\n")}
    </pre>
  );
}
