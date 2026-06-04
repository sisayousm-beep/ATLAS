// Tiny inline-SVG line chart for the Phase 5 nation dashboard.
//
// No chart library (PixiJS drives the map; this is plain SVG). It draws one
// series auto-scaled to its own min/max, with a faint filled area under the line.
// The viewBox is fixed and stretched to the container, so the stroke uses
// non-scaling-stroke to stay an even width.

const W = 100;
const H = 40;
const PAD = 2;

export function LineChart({ data, color = "#5fb98a" }: { data: number[]; color?: string }) {
  if (data.length === 0) {
    return <div className="chart-empty">데이터 수집 중…</div>;
  }

  const min = Math.min(...data);
  const max = Math.max(...data);
  const span = max - min || Math.abs(max) || 1;
  const n = data.length;
  const x = (i: number) => (n === 1 ? W / 2 : (i / (n - 1)) * W);
  const y = (v: number) => H - PAD - ((v - min) / span) * (H - 2 * PAD);

  const line = data.map((v, i) => `${x(i).toFixed(1)},${y(v).toFixed(1)}`).join(" ");
  const area = `${x(0).toFixed(1)},${H} ${line} ${x(n - 1).toFixed(1)},${H}`;

  return (
    <svg className="chart" viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="none" role="img">
      <polygon points={area} fill={color} fillOpacity={0.12} />
      <polyline
        points={line}
        fill="none"
        stroke={color}
        strokeWidth={1.5}
        strokeLinejoin="round"
        vectorEffect="non-scaling-stroke"
      />
    </svg>
  );
}
