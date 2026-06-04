// Region & Nation Inspector (UI roadmap Phase 3).
//
// Renders the details panel for whatever the player has selected on the map.
// All figures are read straight off the current world snapshot — region facts
// come from the region itself, nation facts are aggregated across the nation's
// regions and its finance/politics/military rows.

import type { Selection } from "../map/AtlasMap";
import type { WorldView } from "../types";
import {
  climateLabel,
  goodLabel,
  governmentLabel,
  terrainLabel,
} from "../i18n/labels";

interface Props {
  world: WorldView;
  selection: Selection;
  onSelectRegion: (id: number) => void;
  onSelectNation: (id: string) => void;
  onClose: () => void;
}

const fmt = (n: number) => Math.round(n).toLocaleString();
const swatch = (color: number) => `#${color.toString(16).padStart(6, "0")}`;

/** A labelled meter bar (0..1), reused for infrastructure and stability. */
function Meter({ value }: { value: number }) {
  return (
    <span className="insp-meter">
      <span className="insp-meter-track">
        <span className="insp-meter-fill" style={{ width: `${Math.round(value * 100)}%` }} />
      </span>
      {Math.round(value * 100)}%
    </span>
  );
}

export function Inspector({ world, selection, onSelectRegion, onSelectNation, onClose }: Props) {
  if (!selection) return null;

  if (selection.kind === "region") {
    const region = world.regions.find((r) => r.id === selection.id);
    if (!region) return null;
    const nation = world.nations.find((n) => n.id === region.nationId);

    return (
      <div className="inspector">
        <header className="insp-head">
          <span className="insp-kind">지역</span>
          <h2>{region.name}</h2>
          <button className="insp-close" onClick={onClose} aria-label="닫기">
            ✕
          </button>
        </header>

        {nation && (
          <button className="insp-owner" onClick={() => onSelectNation(nation.id)}>
            <span className="swatch swatch-sm" style={{ background: swatch(nation.color) }} />
            {nation.name}
          </button>
        )}

        <dl className="insp-stats">
          <div>
            <dt>인구</dt>
            <dd>{region.population.toLocaleString()}</dd>
          </div>
          <div>
            <dt>지형</dt>
            <dd>{terrainLabel(region.terrain)}</dd>
          </div>
          <div>
            <dt>기후</dt>
            <dd>{climateLabel(region.climate)}</dd>
          </div>
          <div>
            <dt>인프라</dt>
            <dd>
              <Meter value={region.infrastructure} />
            </dd>
          </div>
        </dl>

        <h3 className="insp-sub">자원</h3>
        {region.resources.length > 0 ? (
          <ul className="insp-list">
            {region.resources.map((r) => (
              <li key={r.good} className="insp-res">
                <span>{goodLabel(r.good)}</span>
                <span className="insp-res-mult">×{r.abundance.toFixed(1)}</span>
              </li>
            ))}
          </ul>
        ) : (
          <div className="insp-empty">부존자원 없음</div>
        )}
      </div>
    );
  }

  // --- nation ---
  const nation = world.nations.find((n) => n.id === selection.id);
  if (!nation) return null;
  const regions = world.regions.filter((r) => r.nationId === nation.id);
  const population = regions.reduce((s, r) => s + r.population, 0);
  const fin = world.finance.find((f) => f.nationId === nation.id);
  const pol = world.politics.find((p) => p.nationId === nation.id);
  const mil = world.military.find((m) => m.nationId === nation.id);
  const corps = world.corporations.filter((c) => c.nationId === nation.id);

  return (
    <div className="inspector">
      <header className="insp-head">
        <span className="insp-kind">국가</span>
        <h2>
          <span className="swatch swatch-sm" style={{ background: swatch(nation.color) }} />
          {nation.name}
        </h2>
        <button className="insp-close" onClick={onClose} aria-label="닫기">
          ✕
        </button>
      </header>

      <dl className="insp-stats">
        <div>
          <dt>인구</dt>
          <dd>{population.toLocaleString()}</dd>
        </div>
        {pol && (
          <div>
            <dt>정부</dt>
            <dd>{governmentLabel(pol.government)}</dd>
          </div>
        )}
        {pol && (
          <div>
            <dt>안정도</dt>
            <dd>
              <Meter value={pol.stability} />
            </dd>
          </div>
        )}
        {fin && (
          <div>
            <dt>부채</dt>
            <dd className={fin.debt > 0 ? "neg" : ""}>
              {fin.debt > 0 ? `−${fmt(fin.debt)}` : "—"}
            </dd>
          </div>
        )}
        {fin && (
          <div>
            <dt>물가상승</dt>
            <dd>{(fin.inflation * 100).toFixed(1)}%</dd>
          </div>
        )}
        {fin && (
          <div>
            <dt>정책금리</dt>
            <dd>{(fin.policyRate * 100).toFixed(1)}%</dd>
          </div>
        )}
        {mil && (
          <div>
            <dt>군사력</dt>
            <dd className={mil.atWar ? "neg" : ""}>
              {mil.atWar ? "⚔ " : ""}
              {fmt(mil.power)}
            </dd>
          </div>
        )}
      </dl>

      <h3 className="insp-sub">지역 ({regions.length})</h3>
      <ul className="insp-list">
        {regions.map((r) => (
          <li key={r.id}>
            <button className="insp-link" onClick={() => onSelectRegion(r.id)}>
              <span>{r.name}</span>
              <span className="insp-res-mult">{r.population.toLocaleString()}</span>
            </button>
          </li>
        ))}
      </ul>

      <h3 className="insp-sub">기업 ({corps.length})</h3>
      <ul className="insp-list">
        {corps.map((c) => (
          <li key={c.name} className="insp-res">
            <span>{c.name}</span>
            <span className={`insp-res-mult ${c.capital < 0 ? "neg" : ""}`}>
              {c.industries.map(goodLabel).join(", ")}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
