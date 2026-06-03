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
4. ~~Phase 4 — Finance, Central Bank~~ ✅
5. ~~Phase 5 — Politics, Diplomacy~~ ✅
6. **Phase 6** — War  ← *current*
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

### Phase 5 — Politics & Diplomacy

- **Legitimacy & stability** — each government represents some ideologies and not
  others (design §13); the share of the population it represents is its *support*.
  Stability eases monthly toward how content the people are (happiness) and how
  well they are represented (support).
- **Unrest & revolution** — low stability is open unrest (strikes, riots). When it
  stays deep for several months the largest ideological bloc seizes power — a
  revolution or coup that installs its own government (conservatives crown a
  monarch, militarists raise a junta, …) and resets stability at a honeymoon level.
- **Politics → economy** — unrest erodes tax compliance, so an unstable state
  collects less, borrows more and (via Phase 4) inflates — which depresses
  happiness and feeds the unrest. The political and monetary loops are coupled.
- **Diplomacy** — every nation pair holds a relation that eases toward government
  compatibility plus a *commercial-peace* bonus from how much the two trade
  (design §14): heavy commerce can thaw even an ideological rivalry. The score maps
  to a status — ally, neutral, rival, hostile.
- **Diplomacy → politics** — a friendly neighbourhood lifts a nation's prestige and
  steadies its regime; hostility erodes both, feeding back into §13 stability.

Emergent so far: Aurelia's liberal democracy represents its people and stays calm;
Khoresan's autocracy sits on a poorer, inflation-bitten populace, so unrest runs
hot and a coup is never far off. Commerce binds the two industrial powers (Aurelia,
Nordheim) into an alliance across the democracy/monarchy divide, while the
ideological gulf leaves Aurelia and Khoresan rivals.

Deferred from design §13–§14 (post-MVP): organised parties and elections, explicit
treaties/tariffs/sanctions as player actions, alliance blocs.

### Phase 6 — War

War is the *extension* of the economy, not its purpose (design §2, §15):

- **Military from the economy** — each nation funds a standing army out of its
  treasury (산업력·경제력); the spending buys *strength*, eroded by peacetime upkeep.
  A nation's combat *power* is that strength plus the manpower of its soldier pops
  (병력), scaled by technology (기술력) and degraded by war exhaustion (보급·사기).
  A broke nation cannot arm — guns rest on butter.
- **Who fights** — only the militant reach for the sword (juntas above autocracies
  above monarchies above democracies), and only against a rival they already loathe
  and decisively out-power. Commerce-bound powers stay at peace.
- **Prosecution** — fighting grinds both armies down by attrition, burns treasury
  the state must often borrow (so war prints money and shows up as inflation — Phase
  4), and wears out the home front. A spent or hopeless side sues for peace.
- **Settlement** — the victor takes prestige and reparations; the loser's standing
  and regime stability are dragged down (§15 → §13).
- **War cuts commerce** — while it lasts, trade between belligerents is embargoed
  (Phase 3) and their relation is frozen at its wartime low (Phase 5).

Emergent so far: the surplus-rich industrial powers (Aurelia, Nordheim) field the
strongest militaries while broke Khoresan can barely fund one — and, true to the
design, the world mostly stays at peace: the rivalry between Aurelia and Khoresan is
cold but no aggressor holds a decisive enough edge, so commerce keeps the guns
holstered. War is a tool of last resort the economy rarely affords.

## Run

```bash
cd backend
cargo run      # real-time sim: 1s = 1 day; monthly reports: nations, prices, firms, trade, finance, politics, military
cargo test     # headless checks: world coherence + production/prices + firms/trade + finance + politics/diplomacy + war

cd ../frontend
npm install
npm run dev     # PixiJS map + live-style market panel
```
