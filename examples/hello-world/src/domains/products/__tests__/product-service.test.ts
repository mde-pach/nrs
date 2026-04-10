import { describe, it, expect, beforeEach, afterAll } from "vitest";
import { getTestDb, cleanTestDb, destroyTestDb } from "@/shared/test-db";
import {
  getProducts,
  getProductById,
  reserveStock,
} from "../services/product-service";
import { ProductNotFoundError, InsufficientInventoryError } from "../errors";

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

describe("getProducts", () => {
  it("returns non-archived products", async () => {
    await db.product.createMany({
      data: [
        {
          name: "Visible",
          description: "",
          priceInCents: 1000,
          stockCount: 10,
          categoryId,
        },
        {
          name: "Archived",
          description: "",
          priceInCents: 1000,
          stockCount: 10,
          categoryId,
          archived: true,
        },
      ],
    });

    const products = await getProducts(db);
    expect(products).toHaveLength(1);
    expect(products[0].name).toBe("Visible");
  });
});

describe("getProductById", () => {
  it("throws ProductNotFoundError for missing product", async () => {
    await expect(getProductById(db, "nonexistent")).rejects.toThrow(
      ProductNotFoundError,
    );
  });
});

describe("reserveStock", () => {
  it("decrements stock and returns updated product", async () => {
    const product = await db.product.create({
      data: {
        name: "Item",
        description: "",
        priceInCents: 500,
        stockCount: 10,
        categoryId,
      },
    });

    const result = await reserveStock(db, product.id, 3);
    expect(result.stockCount).toBe(7);

    const dbProduct = await db.product.findUnique({
      where: { id: product.id },
    });
    expect(dbProduct?.stockCount).toBe(7);
  });

  it("throws InsufficientInventoryError when stock is too low", async () => {
    const product = await db.product.create({
      data: {
        name: "Scarce",
        description: "",
        priceInCents: 500,
        stockCount: 2,
        categoryId,
      },
    });

    await expect(reserveStock(db, product.id, 5)).rejects.toThrow(
      InsufficientInventoryError,
    );

    const dbProduct = await db.product.findUnique({
      where: { id: product.id },
    });
    expect(dbProduct?.stockCount).toBe(2);
  });
});
