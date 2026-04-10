import { describe, it, expect, beforeEach, afterAll } from "vitest";
import { getTestDb, cleanTestDb, destroyTestDb } from "@/shared/test-db";
import { checkout } from "../services/checkout-service";
import { EmptyCartError } from "../errors";
import { InsufficientInventoryError } from "../../products/errors";

const db = getTestDb();
let categoryId: string;

beforeEach(async () => {
  await cleanTestDb(db);
  const category = await db.category.create({ data: { name: "Test" } });
  categoryId = category.id;
});

afterAll(async () => {
  await destroyTestDb();
});

describe("checkout", () => {
  it("creates an order from cart and decrements inventory", async () => {
    const product = await db.product.create({
      data: {
        name: "Headphones",
        description: "Wireless",
        priceInCents: 9999,
        stockCount: 50,
        categoryId,
      },
    });

    await db.cart.create({
      data: {
        customerId: "customer-1",
        expiresAt: new Date(Date.now() + 86400000),
        items: { create: [{ productId: product.id, quantity: 2 }] },
      },
    });

    const order = await checkout(db, "customer-1");

    expect(order.status).toBe("pending");
    expect(order.totalInCents).toBe(19998);
    expect(order.lines).toHaveLength(1);
    expect(order.lines[0].productName).toBe("Headphones");
    expect(order.lines[0].priceInCents).toBe(9999);
    expect(order.lines[0].quantity).toBe(2);

    const dbProduct = await db.product.findUnique({
      where: { id: product.id },
    });
    expect(dbProduct?.stockCount).toBe(48);

    const cart = await db.cart.findUnique({
      where: { customerId: "customer-1" },
    });
    expect(cart).toBeNull();
  });

  it("throws EmptyCartError when no cart exists", async () => {
    await expect(checkout(db, "no-cart-customer")).rejects.toThrow(
      EmptyCartError,
    );
  });

  it("throws InsufficientInventoryError and does not create order", async () => {
    const product = await db.product.create({
      data: {
        name: "Rare Item",
        description: "",
        priceInCents: 5000,
        stockCount: 1,
        categoryId,
      },
    });

    await db.cart.create({
      data: {
        customerId: "customer-2",
        expiresAt: new Date(Date.now() + 86400000),
        items: { create: [{ productId: product.id, quantity: 5 }] },
      },
    });

    await expect(checkout(db, "customer-2")).rejects.toThrow(
      InsufficientInventoryError,
    );

    const orders = await db.order.findMany({
      where: { customerId: "customer-2" },
    });
    expect(orders).toHaveLength(0);

    const dbProduct = await db.product.findUnique({
      where: { id: product.id },
    });
    expect(dbProduct?.stockCount).toBe(1);
  });
});
