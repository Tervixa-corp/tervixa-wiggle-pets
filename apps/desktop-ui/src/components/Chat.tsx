import { useEffect, useRef } from "react";
import type { StreamItem } from "../App";

export default function Chat({ items, busy }: { items: StreamItem[]; busy: boolean }) {
  const streamRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = streamRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [items.length]);

  return (
    <div className="chat">
      <div className="stream" ref={streamRef}>
        {items.map((item, i) => (
          <div key={i} className={`msg ${item.kind}`}>
            {item.role && <div className="role">{item.role}</div>}
            {item.title && (
              <div className="role">
                {item.ok ? "✓" : "✗"} {item.title}
              </div>
            )}
            {item.mono ? (
              <div className="text mono">{item.text}</div>
            ) : (
              <div className="text">{item.text}</div>
            )}
          </div>
        ))}
        {busy && <div className="msg system">…working</div>}
      </div>
    </div>
  );
}
