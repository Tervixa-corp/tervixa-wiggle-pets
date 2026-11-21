export type RailView = "sessions" | "files" | "memory" | "mcp";

interface Props {
  view: RailView;
  onView: (v: RailView) => void;
  auxOpen: boolean;
  onAux: () => void;
}

const VIEWS: { id: RailView; glyph: string; hint: string }[] = [
  { id: "sessions", glyph: "»", hint: "sessions" },
  { id: "files", glyph: "f", hint: "files" },
  { id: "memory", glyph: "m", hint: "memory" },
  { id: "mcp", glyph: "x", hint: "mcp servers" },
];

export default function Rail({ view, onView }: Props) {
  return (
    <div className="rail">
      {VIEWS.map((v) => (
        <button
          key={v.id}
          className={view === v.id ? "active" : ""}
          title={v.hint}
          onClick={() => onView(v.id)}
        >
          {v.glyph}
        </button>
      ))}
      <div className="gap" />
    </div>
  );
}
