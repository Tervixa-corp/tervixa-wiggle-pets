interface Props {
  title: string;
  onSettings: () => void;
  onToggleAux: () => void;
}

export default function TitleBar({ title, onSettings, onToggleAux }: Props) {
  return (
    <div className="titlebar" data-tauri-drag-region>
      <span className="mark">TERVIXA WIGGLE</span>
      <span>{title.split(" ")[1] ?? ""}</span>
      <span className="spacer" />
      <button className="item" onClick={onToggleAux} style={{ width: "auto", padding: "0 8px" }}>
        aux
      </button>
      <button className="item" onClick={onSettings} style={{ width: "auto", padding: "0 8px" }}>
        settings
      </button>
    </div>
  );
}
