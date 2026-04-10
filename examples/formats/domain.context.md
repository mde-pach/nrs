# Domain Context — Products

## Business Purpose

The product catalog is the central source of truth for everything the company sells. It serves both the customer-facing storefront and the back-office.

## Core Concepts

- **Product**: A sellable item with a name, description, price, and available stock. Belongs to exactly one category.
- **Category**: A grouping of products for navigation and filtering. Categories form a tree.
- **Inventory**: The available quantity of a product. When inventory reaches zero, the product becomes unavailable but is not removed.
- **Price**: The current selling price. Price history is retained for audit.

## Business Rules

- A product with pending orders cannot be deleted — only archived
- Price changes take effect immediately but do not affect already-placed orders
- Inventory is decremented at order confirmation, not at cart addition
- Products with zero inventory remain visible but marked as unavailable

## Domain Relations

- **Orders**: Orders reference products at the time of purchase. The order stores a snapshot of the product name and price — no live reference.
