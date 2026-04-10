# Domain Context — Products

## Business Purpose

Source of truth for items available for sale. Powers both customer browsing and inventory management.

## Core Concepts

- **Product**: A sellable item with a name, description, price, and available stock
- **Category**: A grouping of products for navigation and filtering

## Business Rules

- A product with pending orders cannot be deleted, only archived
- Price changes do not affect already-placed orders
- Inventory is decremented at order confirmation, not at cart addition
- Products with zero stock remain visible but marked as unavailable

## Domain Relations

- **Orders**: Orders snapshot product name and price at purchase time — no live reference
