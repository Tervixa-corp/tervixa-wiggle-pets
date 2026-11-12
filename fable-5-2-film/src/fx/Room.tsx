import React from "react";
import { AbsoluteFill, useCurrentFrame } from "remotion";
import { theme } from "../theme";
import { progress } from "../components/motion";

/**
 * The room. One L2 plane per act, rendered at z=0 — between the Backdrop and
 * the Thread.
 *
 * Why this file exists: the L2 planes used to live at the top of each scene
 * component, which meant they rendered ABOVE the Thread and, worse, each scene
 * root also painted an opaque `bg`. Both facts hid the film's one continuous
 * instrument for most of its running time. The rule the spec sets down is that
 * paint order is the occlusion — Thread above a scene layer is a build failure —
 * so the room moves down here, where a glyph can still cover the line but a
 * background wash cannot.
 *
 * The planes are keyed to the act boundaries and enter on the same hard cut as
 * their content: no exports, no fades out.
 */

type Plane = {
  /** Global frame the plane appears. Always an act boundary. */
  from: number;
  until: number;
  left: number;
  top: number;
  width: number;
  height: number;
  opacity: number;
};

const PLANES: Plane[] = [
  // Cold open — the far wall the wordmark is set against
  { from: 0, until: 210, left: 300, top: 300, width: 1320, height: 380, opacity: 0.9 },
  // Positioning — the wall behind the text block
  { from: 210, until: 420, left: 170, top: 250, width: 860, height: 600, opacity: 0.85 },
  // Benchmarks — no plane here: the chart builds its own lit well, and two
  // surfaces under one chart reads as a shadow.
  // Proof — no plane here: each vignette carries its own light table.
  // Economics — the ledger slab
  { from: 1200, until: 1500, left: 430, top: 300, width: 1100, height: 400, opacity: 0.9 },
  // Lockup — the floor under the name
  { from: 1500, until: 1800, left: 210, top: 520, width: 1500, height: 320, opacity: 0.85 },
];

export const Room: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <AbsoluteFill style={{ pointerEvents: "none" }}>
      {PLANES.filter((p) => frame >= p.from && frame < p.until).map((p) => {
        const e = progress(frame, p.from, 24);
        return (
          <div
            key={p.from}
            style={{
              position: "absolute",
              left: p.left,
              top: p.top,
              width: p.width,
              height: p.height,
              background: `radial-gradient(ellipse 100% 100% at 50% 50%, ${theme.colors.bgRaise} 0%, ${theme.colors.bgRaise} 54%, rgba(54,59,69,0) 100%)`,
              opacity: p.opacity * e,
            }}
          />
        );
      })}
    </AbsoluteFill>
  );
};
