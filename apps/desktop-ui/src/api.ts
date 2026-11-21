// Typed bridge to the Tauri engine commands.
import { invoke } from "@tauri-apps/api/core";

export interface ProviderView {
  id: string;
  label: string;
  api: string;
  base_url: string;
  models: string[];
  enabled: boolean;
  has_key: boolean;
}

export interface EngineStatus {
  product: string;
  version: string;
  workspace: string;
  skills: number;
  mcp_ready: number;
  providers: ProviderView[];
  default_provider: string | null;
  default_model: string | null;
}

export interface TreeNode {
  name: string;
  dir: boolean;
  children: TreeNode[] | null;
}

export interface MemoryEntryView {
  id: number;
  kind: string;
  content: string;
  pinned: boolean;
  created_at: string;
  score?: number;
}

export interface McpStateView {
  name: string;
  transport: string;
  connected: boolean;
  error: string | null;
  tools: number;
  resources: number;
  prompts: number;
}

export const status = () => invoke<EngineStatus>("engine_status");
export const settings = () =>
  invoke<Record<string, unknown>>("engine_settings");
export const saveSettings = (config: Record<string, unknown>) =>
  invoke<void>("save_settings", { config });
export const sendPrompt = (
  session: string,
  prompt: string,
  agent?: string,
  model?: string,
) =>
  invoke<string>("send_prompt", {
    session,
    prompt,
    agent: agent ?? null,
    model: model ?? null,
  });
export const workspaceTree = () => invoke<TreeNode[]>("list_workspace_tree");
export const readFile = (path: string) =>
  invoke<string>("read_workspace_file", { path });
export const writeFile = (path: string, content: string) =>
  invoke<void>("write_workspace_file", { path, content });
export const undo = () => invoke<string | null>("undo_workspace");
export const redo = () => invoke<string | null>("redo_workspace");
export const memoryList = (limit = 50) =>
  invoke<MemoryEntryView[]>("memory_entries", { limit });
export const memoryAdd = (content: string) =>
  invoke<number>("memory_add", { content });
export const memoryDelete = (id: number) => invoke<void>("memory_delete", { id });
export const memoryPin = (id: number, pinned: boolean) =>
  invoke<void>("memory_pin", { id, pinned });
export const importSkillsClaude = () => invoke<number>("import_skills_claude");
export const importSkillsGithub = (repo: string) =>
  invoke<number>("import_skills_github", { repo });
export const connectMcpAll = () => invoke<McpStateView[]>("connect_mcp_all");
export const mcpCatalog = () =>
  invoke<{ name: string; description: string }[]>("mcp_catalog");
