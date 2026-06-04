// Notification feed (UI roadmap Phase 4 — Strategic HUD).
//
// Shows the most recent events the store derived from the snapshot stream,
// pinned over the map. Newest first.

import { useGameStore } from "../store/gameStore";

export function NotificationFeed() {
  const notifications = useGameStore((s) => s.notifications);
  if (notifications.length === 0) return null;

  return (
    <div className="feed">
      {notifications.slice(0, 6).map((n) => (
        <div key={n.id} className={`feed-item feed-${n.tone}`}>
          <span className="feed-date">
            {n.year}년 {n.month}월
          </span>
          <span className="feed-text">{n.text}</span>
        </div>
      ))}
    </div>
  );
}
