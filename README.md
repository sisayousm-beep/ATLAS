# Project Atlas

**Global Economic Grand Strategy Simulator** — a living civilization engine where economy, politics, industry, technology and finance are the core, not war.

> War is a tool. The economy is the goal.

See [`Project Atlas 설계도.json`](./Project%20Atlas%20%EC%84%A4%EA%B3%84%EB%8F%84.json) for the full design.

## Stack

| Layer         | Tech                  |
| ------------- | --------------------- |
| Simulation    | Rust + `bevy_ecs`     |
| Backend API   | Rust                  |
| Database      | PostgreSQL 17         |
| Frontend      | React + TypeScript    |
| Visualization | PixiJS                |

## Layout

```
backend/   Rust simulation core (bevy_ecs), real-time tick (1s = 1 game day)
frontend/  React + TS + PixiJS client
db/        PostgreSQL schema + migrations
```

## Roadmap (MVP phases)

1. ~~Phase 1 — Region, Nation, POP, Resources~~ ✅
2. **Phase 2** — Production, Market, Prices  ← *current*
3. Phase 3 — Corporation AI, Trade
4. Phase 4 — Finance, Central Bank
5. Phase 5 — Politics, Diplomacy
6. Phase 6 — War
7. Phase 7 — Nation AI

### Phase 2 — Production & Market

- **Deposits** — regions hold raw goods (iron ore, coal, oil) at varying richness.
- **Extraction** — laborers mine raw goods, scaled by deposit richness × infrastructure.
- **Manufacturing** — engineers run recipes up the three tiers:
  `iron ore + coal → iron → steel`, `oil → plastic`, `steel + plastic → car`.
- **Market** — one global price per good, formed daily from supply vs demand
  (no central planner). Surplus goods fall toward a floor; scarce goods rise
  toward a ceiling around their base price.

Emergent so far: Nordheim industrializes (cheap steel), Khoresan pumps oil,
cars stay scarce and expensive.

## Run

```bash
cd backend
cargo run      # real-time sim: 1 second = 1 day, monthly reports incl. prices
cargo test     # headless checks: world coherence + production/price movement

cd ../frontend
npm install
npm run dev     # PixiJS map + live-style market panel
```
