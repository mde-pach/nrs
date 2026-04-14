# Hello World — Backlog

Product roadmap for the Hello World Commerce app. Tickets are grouped by area, each with acceptance criteria. Pick one, build it.

## How to work a ticket

```
git checkout -b story/<handle>-HW-<n>
npm install && npm run db:setup
npm run dev
```

Local branches only — nothing is pushed, nothing is merged. One ticket per branch. Reset between tickets with `git checkout . && git clean -fd && rm -f prisma/dev.db && npm run db:setup`.

---

## Catalog

### HW-1 — Sort products on the home page
As a shopper, I want to sort the product list by name or price so I can browse in an order that suits me.
- `?sort=name-asc|name-desc|price-asc|price-desc` on `/`
- Default sort is `name-asc`
- Invalid values fall back to the default
- Integration test covers each sort direction

### HW-2 — Filter products by category
As a shopper, I want to narrow the product list to a single category so I can find what I'm looking for faster.
- `?category=<categoryId>` on `/`
- Unknown category ids return an empty list, not an error
- Categories render as a navigation strip above the product grid
- Combines with `sort` from HW-1
- Integration test with seeded categories

### HW-3 — Search products by name
As a shopper, I want to search products by keyword so I can find items without scrolling.
- `?q=<term>` on `/`, case-insensitive, matches anywhere in the name
- Empty `q` behaves as no filter
- Integration test with seeded fixtures

### HW-4 — Low-stock indicator
As a shopper, I want to know when an item is about to sell out so I can decide whether to buy now.
- Product cards show a "Low stock" badge when `stockCount < 5`
- Out-of-stock products (`stockCount === 0`) show "Unavailable" and the Add-to-Cart control is disabled
- Component test covers both states

### HW-5 — Product detail page
As a shopper, I want a dedicated page per product so I can read the full description before adding to cart.
- `/products/[id]` renders name, description, price, stock state
- Missing id returns a 404
- Add-to-cart button on this page reuses the same cart logic as the grid card

---

## Cart & Checkout

### HW-6 — Cancel a pending order
As a customer, I want to cancel an order that hasn't shipped yet so I can change my mind without a refund process.
- `POST /api/orders/:id/cancel`
- Only orders with status `pending` can be cancelled; any other status returns a typed error mapped to `409 Conflict`
- Cancelled orders restore reserved inventory (if inventory was reserved for this order)
- Integration test covers: pending → cancelled success, confirmed → error, non-existent id → 404

### HW-7 — Discount codes at checkout
As a customer, I want to apply a promo code at checkout so I can get the advertised discount.
- New concept: a discount code with either a fixed-cents amount or a percentage, an optional expiry, and an optional usage cap
- Seed a handful of codes for testing
- `POST /api/orders` accepts `{ discountCode?: string }`; invalid/expired/exhausted codes return a typed error
- Order stores the code applied and the discounted total
- Integration tests: valid fixed, valid percentage, expired, exhausted, unknown

### HW-8 — Checkout with inventory reservation
As a customer, I want checkout to either succeed fully or fail without half-charging me, so my cart stays consistent.
- `POST /api/orders` reads the cart, verifies stock for every line, decrements stock, creates the order, clears the cart — in a single transaction
- If any line has insufficient stock, the whole operation rolls back and returns a typed error listing the offending products
- Integration tests: success path, partial stock failure, empty cart

### HW-9 — Refund a confirmed order
As a customer, I want to request a refund on an order so I can return items I no longer want.
- `POST /api/orders/:id/refund`
- Only `confirmed` / `shipped` / `delivered` orders can be refunded
- Refund restores inventory for every line
- Order status becomes `refunded`
- Integration tests for each allowed status and one negative case

### HW-10 — Guest checkout
As a first-time visitor, I want to check out without creating an account so I can buy quickly.
- Checkout accepts an anonymous session id in place of a customer id
- Guest orders persist the email captured at checkout
- Future lookup of a guest order requires the order id + email
- Integration test covers the full guest flow

---

## Customer

### HW-11 — Wishlist
As a shopper, I want to save products for later so I can come back to them.
- Add / remove a product from the customer's wishlist
- Wishlist is visible on a `/wishlist` page
- Adding an out-of-stock product is allowed; the wishlist surfaces the stock state
- Integration tests for add, remove, and listing

### HW-12 — Order history
As a customer, I want to see the orders I've placed so I can track and reorder.
- `/orders` lists the current customer's orders, most recent first
- Each row links to `/orders/[id]` with the full order detail
- Pagination via cursor
- Integration test seeded with multiple orders

---

## Admin

### HW-13 — Archive a product
As an admin, I want to remove a product from the catalog without losing its history, so past orders still display correctly.
- `DELETE /api/products/:id` archives when the product has any associated order line, otherwise deletes
- Archived products do not appear in the public listings or search
- Order history continues to render archived products using the stored snapshot
- Integration tests: delete with no orders, delete with orders → archive, already-archived

### HW-14 — Bulk stock update
As an admin, I want to update stock counts for many products at once so inventory sync is fast.
- `POST /api/admin/products/stock` accepts an array of `{ id, stockCount }`
- The endpoint is transactional — any invalid id rolls back the whole batch
- Response reports per-item success/failure when validation passes but a downstream step fails
- Integration test with mixed valid / invalid payloads
