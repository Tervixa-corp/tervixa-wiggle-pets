import { useState } from "react";
import type { TreeNode, EngineStatus, MemoryEntryView, McpStateView } from "../api";
import type { RailView } from "./Rail";

interface Props {
  view: RailView;
  tree: TreeNode[];
  status: EngineStatus | null;
  onAux: (tab: "memory" | "raw") => void;
}

export default function Sidebar({ view, tree, status, onAux }: Props) {
  const [openDirs, setOpenDirs] = useState<Set<string>>(new Set());

  if (view === "files") {
    return (
      <div className="sidebar tree">
        <h3>Workspace</h3>
        <Tree nodes={tree} depth={0} openDirs={openDirs} setOpenDirs={setOpenDirs} />
      </div>
    );
  }

  if (view === "memory") {
    return (
      <div className="sidebar">
        <h3>Memory Vault</h3>
        <button className="item" onClick={() => onAux("memory")}>
          Open memory pane
        </button>
      </div>
    );
  }

  if (view === "mcp") {
    return (
      <div className="sidebar">
        <h3>MCP Servers</h3>
        <button className="item" onClick={() => onAux("raw")}>
          Connection log
        </button>
      </div>
    );
  }

  // sessions
  return (
    <div className="sidebar">
      <h3>Providers</h3>
      {(status?.providers ?? []).map((p) => (
        <button key={p.id} className="item" title={`${p.base_url} · ${p.api}`}>
          {p.has_key ? "● " : "○ "}
          {p.label || p.id}
        </button>
      ))}
      <h3>Skills</h3>
      <button className="item">{status?.skills ?? 0} loaded</button>
    </div>
  );
}

function Tree({
  nodes,
  depth,
  openDirs,
  setOpenDirs,
}: {
  nodes: TreeNode[];
  depth: number;
  openDirs: Set<string>;
  setOpenDirs: (v: Set<string>) => void;
}) {
  return (
    <>
      {nodes.map((n) => {
        const path = n.name;
        if (n.dir) {
          const open = openDirs.has(path + depth);
          return (
            <div key={path}>
              <button
                className="item dir"
                onClick={() => {
                  const next = new Set(openDirs);
                  if (open) next.delete(path + depth);
                  else next.add(path + depth);
                  setOpenDirs(next);
                }}
              >
                {open ? "▾" : "▸"} {n.name}
              </button>
              {open && n.children && (
                <div style={{ paddingLeft: 12 }}>
                  <Tree
                    nodes={n.children}
                    depth={depth + 1}
                    openDirs={openDirs}
                    setOpenDirs={setOpenDirs}
                  />
                </div>
              )}
            </div>
          );
        }
        return (
          <button key={path} className="item file">
            {n.name}
          </button>
        );
      })}
    </>
  );
}

export type { MemoryEntryView, McpStateView };
