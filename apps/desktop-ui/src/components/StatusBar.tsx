import type { EngineStatus } from "../api";

export default function StatusBar({ status, busy }: { status: EngineStatus | null; busy: boolean }) {
  return (
    <div className="statusbar">
      <span>{busy ? "running" : "idle"}</span>
      <span>{status?.default_provider ?? "no provider"}</span>
      <span>{status?.default_model ?? "—"}</span>
      <span className="grow" />
      <span>skills {status?.skills ?? 0}</span>
      <span>mcp {status?.mcp_ready ?? 0} ready</span>
      <span>{status?.workspace ?? "…"}</span>
    </div>
  );
}
