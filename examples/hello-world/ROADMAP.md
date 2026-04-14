# Hello World — Test Stories

A scratchpad of user stories for exercising NRS with Claude Code on a realistic codebase. Pick one, build it with an agent, and pay attention to what NRS does around you — which context files get loaded, which gaps surface, whether the loop actually helps.

## How to use this

1. **Branch off this testing branch — do not commit back to it.**
   ```
   git checkout -b story/<your-handle>-<story-slug>
   ```
   Your branch is a local scratchpad. No push, no PR. The point is to see the tool in action, not to ship the code.

2. **Bring the CLI on PATH.**
   ```
   cargo install --path ../../cli          # from this directory
   # or use the repo-local build:
   alias nrs=$(git rev-parse --show-toplevel)/cli/target/release/nrs
   ```

3. **Seed the app.**
   ```
   npm install
   npm run db:setup
   ```

4. **Open Claude Code from this directory** and ask it to pick up one of the stories below. The session will fire the NRS hooks automatically (context layer injection, gap observation, guard on generated files).

5. **Observe, don't just ship.** After the task:
   - `nrs gap summary` — what gaps did the observer detect?
   - Inspect `nrs.gaps.md` — which patterns (excessive-reads, no-context, re-reads, backtracking, user-correction) fired?
   - Did Claude read the right `*.context.md` files before acting? Check the transcript or logs.
   - Was the existing context sufficient? Did you end up explaining the same thing twice?

6. **Reset between stories.**
   ```
   git checkout . && git clean -fd
   rm -f nrs.gaps.md
   npm run db:setup
   ```

## What a good run looks like

- The agent loads `src/domains/<domain>/CLAUDE.md` before touching that domain.
- If the story needs a business rule that isn't in context, `nrs.gaps.md` accumulates a `missing-context` or `missing-pattern` row — ideally before you have to correct the agent.
- For cross-domain work, sub-agents fire with their own layer orientation (`SubagentStart` hook).
- The guard blocks direct edits to `CLAUDE.md` and redirects you to `*.context.md`.

---

## Stories

### A. Context-sufficient (happy path — the loop should feel invisible)

**A1. Sort products on the home page.** Add a `?sort=price|name` query param on `/` and wire the product listing to respect it. Integration test the sort order. Touches `src/app/page.tsx` and `src/domains/products/services/product-service.ts`.

**A2. Low-stock badge on product cards.** Products with `stockCount < 5` render a "Low stock" badge. Decide: is "low stock threshold" a product concern that belongs in `domain.context.md`, or a UI concern? Let the agent propose.

### B. Single domain, a rule to enforce that isn't yet wired

**B1. Cancel a pending order.** `POST /api/orders/:id/cancel`. Orders' `domain.context.md` already says "cancellable only while pending" — enforce it, return a typed error for other statuses, add an integration test.

**B2. Archive instead of delete.** Products' context says "a product with pending orders cannot be deleted, only archived." Add a delete endpoint that enforces this rule (archive when blocked, delete otherwise). The `archived` column exists on the schema; nothing uses it yet.

### C. Cross-domain orchestration

**C1. Refund a confirmed order.** Orders domain says refunds restore inventory. Add `POST /api/orders/:id/refund` that updates the order status and calls back into the products service to increment stock. Watch for: does the agent route the cross-domain call through the service boundary (per Orders' implementation context) or reach into the other domain's repository?

**C2. Full checkout flow with inventory reservation.** `POST /api/orders` should create an order from the caller's cart, reserve stock for every line in one transaction, and clear the cart. If any line has insufficient stock, the whole checkout fails atomically. `checkout-service.ts` exists but is not wired end-to-end.

### D. Context gaps — these stories are designed to surface missing context

**D1. Discount codes at checkout.** Introduce a `DiscountCode` concept (fixed-amount or percentage). Checkout must record the applied code and the discounted total. No context exists for this today. Expected behaviour: the agent should propose where the concept lives (new domain? extension of Orders?) and write the context before the code. If it just dives into code, that's a gap in our operating rules.

**D2. Wishlist.** Customers save products for later. No domain exists. Does the agent propose a new domain with its own `domain.context.md`, or stuff it into an existing one? Capture the reasoning.

**D3. Guest checkout.** The `Order` model has a `customerId`. Can an anonymous customer check out? No context covers this. Gap. Watch whether the agent surfaces the ambiguity before writing code.

### E. Context conflict — a business-rule change

**E1. Reserve inventory on cart add.** Change the rule from "stock is checked only at order confirmation" to "adding an item to the cart reserves stock for 15 minutes, then releases automatically." Both `src/domains/products/domain.context.md` and `src/domains/orders/domain.context.md` need updating before the code changes. Test whether the agent updates context first (per the Propose-First rule) or jumps to code.

---

## Feedback

Not every friction belongs in this file. If you hit a tool-level gap (e.g., a pattern the observer should detect but didn't), open an issue on the main nrs repo with the transcript and a short description. Context-file gaps stay local on your branch — they're exactly what the loop is supposed to surface.
