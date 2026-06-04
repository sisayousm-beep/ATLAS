// Left menu (UI roadmap Phase 4 — Strategic HUD).
//
// The four top-level screens of the strategy client. Selecting one swaps the
// content of the right-hand panel.

export type HudTab = "overview" | "economy" | "trade" | "politics";

const TABS: { id: HudTab; label: string; icon: string }[] = [
  { id: "overview", label: "개요", icon: "🌐" },
  { id: "economy", label: "경제", icon: "💹" },
  { id: "trade", label: "무역", icon: "🚢" },
  { id: "politics", label: "정치", icon: "🏛" },
];

export function LeftNav({ active, onSelect }: { active: HudTab; onSelect: (t: HudTab) => void }) {
  return (
    <nav className="leftnav">
      {TABS.map((t) => (
        <button
          key={t.id}
          className={`nav-item ${active === t.id ? "active" : ""}`}
          onClick={() => onSelect(t.id)}
        >
          <span className="nav-icon">{t.icon}</span>
          <span className="nav-label">{t.label}</span>
        </button>
      ))}
    </nav>
  );
}
