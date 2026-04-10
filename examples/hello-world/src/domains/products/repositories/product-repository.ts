import { PrismaClient } from "@prisma/client";
import type { Product } from "../types";

export async function listProducts(
  db: PrismaClient,
  options?: { cursor?: string; take?: number },
): Promise<Product[]> {
  const take = options?.take ?? 20;
  const records = await db.product.findMany({
    where: { archived: false },
    take,
    ...(options?.cursor
      ? { skip: 1, cursor: { id: options.cursor } }
      : {}),
    orderBy: { createdAt: "desc" },
  });

  return records.map(toProduct);
}

export async function findProductById(
  db: PrismaClient,
  id: string,
): Promise<Product | null> {
  const record = await db.product.findUnique({ where: { id } });
  return record ? toProduct(record) : null;
}

export async function decrementStock(
  db: PrismaClient,
  productId: string,
  quantity: number,
): Promise<void> {
  await db.product.update({
    where: { id: productId },
    data: { stockCount: { decrement: quantity } },
  });
}

export async function incrementStock(
  db: PrismaClient,
  productId: string,
  quantity: number,
): Promise<void> {
  await db.product.update({
    where: { id: productId },
    data: { stockCount: { increment: quantity } },
  });
}

function toProduct(record: {
  id: string;
  name: string;
  description: string;
  priceInCents: number;
  stockCount: number;
  categoryId: string;
  archived: boolean;
}): Product {
  return {
    id: record.id,
    name: record.name,
    description: record.description,
    priceInCents: record.priceInCents,
    stockCount: record.stockCount,
    categoryId: record.categoryId,
    archived: record.archived,
  };
}
