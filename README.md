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
2. ~~Phase 2 — Production, Market, Prices~~ ✅
3. ~~Phase 3 — Corporation AI, Trade~~ ✅
4. **Phase 4** — Finance, Central Bank  ← *current*
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

### Phase 3 — Corporations & Trade

- **Corporations** — production now runs through profit-seeking firms (design §8).
  Each firm specialises in one good, claims engineer labour in its home region,
  buys its inputs and sells its output on the market, and keeps the margin as
  capital. They answer to their books, not the state.
- **Corporate AI** — monthly, a profitable firm pays corporate tax to its nation
  and reinvests by hiring; a loss-making firm sheds workers. Firms grow and
  shrink on their own.
- **Trade / logistics** — goods rarely sit where the factory that needs them is,
  so a logistics layer ships each good from surplus regions to deficit ones
  (design §10), capped by infrastructure with a transport loss. Cross-border
  flows land on each nation's trade balance.

Emergent so far: oil-poor industrial regions import oil and ore and export iron
and cars; Khoresan runs a steep import deficit feeding others' factories; car
plants are the most profitable firms while overbuilt steelworks run at a loss and
lay off staff.

### Phase 4 — Finance & Central Bank

- **Central bank** — every nation runs one (design §11): it sets a policy
  interest rate and tracks the broad money it has issued.
- **Government finance** — monthly, a government spends on its people (welfare,
  services, admin) and services its debt. Tax and corporate tax fund it; any
  shortfall is borrowed and the central bank prints the money, so debt and the
  money supply grow together. A surplus is run down against the debt.
- **Inflation** — no longer a constant: it emerges from monetary growth, so
  monetising a deficit shows up as inflation a month later.
- **Monetary policy** — each bank leans its rate against the inflation gap
  (Taylor-style), raising when inflation runs hot and cutting toward the floor
  when money is stable.
- **Transmission** — dear credit cools the real economy: a high policy rate
  throttles how fast firms take on engineers.
- **Crisis flag** — runaway inflation or a sovereign-debt spiral (design §11
  events) raises a financial-crisis warning.

Deferred from design §11 (post-MVP): tradable bond/stock markets, commercial
banks, insurance and pensions.

Emergent so far: corporate-tax-rich nations (Aurelia, Nordheim) run surpluses —
stable money, ~0 inflation, rates cut to the floor — while thin-industry Khoresan
can't cover welfare, monetises the deficit, and its central bank hikes rates near
11% to fight the resulting inflation.

## Run

```bash
cd backend
cargo run      # real-time sim: 1s = 1 day; monthly reports: nations, prices, firms, trade, finance
cargo test     # headless checks: world coherence + production/prices + firms/trade + finance

cd ../frontend
npm install
npm run dev     # PixiJS map + live-style market panel
```
