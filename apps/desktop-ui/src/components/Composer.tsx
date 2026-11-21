import { useState } from "react";

interface Props {
  busy: boolean;
  onSend: (prompt: string) => void;
}

export default function Composer({ busy, onSend }: Props) {
  const [draft, setDraft] = useState("");

  const submit = () => {
    onSend(draft);
    setDraft("");
  };

  return (
    <div className="composer">
      <textarea
        autoFocus
        placeholder="Describe the task… (Enter to send, Shift+Enter for newline)"
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            submit();
          }
        }}
      />
      <button className="send" disabled={busy || !draft.trim()} onClick={submit}>
        {busy ? "…" : "Send"}
      </button>
    </div>
  );
}
