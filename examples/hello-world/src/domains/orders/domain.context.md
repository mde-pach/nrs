# Domain Context — Orders

## Business Purpose

Handles the full lifecycle of a customer purchase: cart, checkout, and fulfillment.

## Core Concepts

- **Cart**: A temporary collection of items a customer intends to buy. One active cart per customer. Expires after inactivity.
- **Order**: A confirmed purchase created from a cart. Immutable once placed.
- **Order Line**: A single item in an order, storing quantity and a snapshot of the product at time of purchase.
- **Order Status**: pending → confirmed → shipped → delivered. Can also be cancelled or refunded.

## Business Rules

- Cart items do not reserve inventory — stock is checked at order confirmation only
- Orders can be cancelled only while pending
- Refunds restore inventory

## Domain Relations

- **Products**: Product availability is checked at confirmation time. Orders store a snapshot, not a live reference.
