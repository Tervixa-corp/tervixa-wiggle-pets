import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import * as api from "./api";
import TitleBar from "./components/TitleBar";
import Rail from "./components/Rail";
import Sidebar from "./components/Sidebar";
import Chat from "./components/Chat";
import Composer from "./components/Composer";
import AuxPane from "./components/AuxPane";
import StatusBar from "./components/StatusBar";
import SettingsModal from "./components/SettingsModal";

export interface StreamItem {
  kind: "user" | "text" | "tool" | "error" | "system";
  role?: string;
  title?: string;
  text: string;
  mono?: boolean;
  ok?: boolean;
}

type AuxTab = "output" | "memory" | "browser" | "raw";
type RailView = "sessions" | "files" | "memory" | "mcp";

export default function App() {
  const [items, setItems] = useState<StreamItem[]>([
    {
      kind: "system",
      text: "Tervixa Wiggle — engine attached. Type a task below.",
    },
  ]);
  const [status, setStatus] = useState<api.EngineStatus | null>(null);
  const [tree, setTree] = useState<api.TreeNode[]>([]);
  const [railView, setRailView] = useState<RailView>("sessions");
  const [auxOpen, setAuxOpen] = useState(true);
  const [auxTab, setAuxTab] = useState<AuxTab>("output");
  const [busy, setBusy] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const session = useRef("desktop-main");

  const refresh = async () => {
    setStatus(await api.status());
    setTree(await api.workspaceTree());
  };

  useEffect(() => {
    refresh().catch(() => setStatus(null));
    let unlisten: (() => void) | undefined;
    listen<Record<string, unknown>>("agent://", (event) => {
      const e = event.payload as Record<string, unknown> & { type: string };
      setItems((prev) => {
        const next = [...prev];
        switch (e.type) {
          case "turn_started":
            next.push({ kind: "system", text: `turn started · ${String(e.agent)}` });
            break;
          case "text_delta":
            if (next.length && next[next.length - 1].kind === "text") {
              next[next.length - 1] = {
                ...next[next.length - 1],
                text: next[next.length - 1].text + String(e.text ?? ""),
              };
            } else {
              next.push({ kind: "text", text: String(e.text ?? ""), role: "wiggle" });
            }
            break;
          case "tool_started":
            next.push({
              kind: "tool",
              title: `${String((e.call as Record<string, unknown>)?.tool ?? "tool")}`,
              text: JSON.stringify((e.call as Record<string, unknown>)?.args ?? {}, null, 2),
              mono: true,
              ok: true,
            });
            break;
          case "tool_finished":
            next.push({
              kind: "tool",
              title: `${String((e.call as Record<string, unknown>)?.tool ?? "tool")} done`,
              text: String((e.call as Record<string, unknown>)?.result ?? ""),
              mono: true,
              ok: e.ok !== false,
            });
            break;
          default:
            break;
        }
        return next.slice(-500);
      });
    }).then((fn) => (unlisten = fn));
    return () => unlisten?.();
  }, []);

  const send = async (prompt: string) => {
    if (!prompt.trim() || busy) return;
    setItems((prev) => [...prev, { kind: "user", text: prompt }]);
    setBusy(true);
    try {
      const reply = await api.sendPrompt(session.current, prompt);
      setItems((prev) => [...prev, { kind: "text", role: "wiggle", text: reply }]);
    } catch (err) {
      setItems((prev) => [...prev, { kind: "error", text: String(err) }]);
    } finally {
      setBusy(false);
      refresh().catch(() => undefined);
    }
  };

  return (
    <div className="app">
      <TitleBar
        title={status ? `${status.product} ${status.version}` : "Tervixa Wiggle"}
        onSettings={() => setShowSettings(true)}
        onToggleAux={() => setAuxOpen((v) => !v)}
      />
      <div className={"body" + (auxOpen ? "" : " no-aux")}>
        <Rail
          view={railView}
          onView={(v) => setRailView(v)}
          auxOpen={auxOpen}
          onAux={() => setAuxOpen((v) => !v)}
        />
        <Sidebar
          view={railView}
          tree={tree}
          status={status}
          onAux={(tab: AuxTab) => {
            setAuxTab(tab);
            setAuxOpen(true);
          }}
        />
        <Chat items={items} busy={busy} />
        {auxOpen && (
          <AuxPane
            tab={auxTab}
            onTab={(t) => setAuxTab(t)}
            items={items}
            status={status}
          />
        )}
      </div>
      <Composer busy={busy} onSend={send} />
      <StatusBar status={status} busy={busy} />
      {showSettings && (
        <SettingsModal
          status={status}
          onClose={() => setShowSettings(false)}
          onSaved={() => refresh().catch(() => undefined)}
        />
      )}
    </div>
  );
}
