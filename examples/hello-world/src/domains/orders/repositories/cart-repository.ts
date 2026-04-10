import type { PrismaClient } from "@prisma/client";
import type { Cart } from "../types";

const CART_TTL_DAYS = 7;

export async function findCartByCustomer(
  db: PrismaClient,
  customerId: string,
): Promise<Cart | null> {
  const record = await db.cart.findUnique({
    where: { customerId },
    include: { items: true },
  });
  return record ? toCart(record) : null;
}

export async function upsertCart(
  db: PrismaClient,
  customerId: string,
  items: { productId: string; quantity: number }[],
): Promise<Cart> {
  const expiresAt = new Date();
  expiresAt.setDate(expiresAt.getDate() + CART_TTL_DAYS);

  const record = await db.cart.upsert({
    where: { customerId },
    create: {
      customerId,
      expiresAt,
      items: { create: items },
    },
    update: {
      expiresAt,
      items: {
        deleteMany: {},
        create: items,
      },
    },
    include: { items: true },
  });

  return toCart(record);
}

export async function deleteCart(
  db: PrismaClient,
  customerId: string,
): Promise<void> {
  await db.cart.deleteMany({ where: { customerId } });
}

function toCart(record: {
  id: string;
  customerId: string;
  expiresAt: Date;
  items: { productId: string; quantity: number }[];
}): Cart {
  return {
    id: record.id,
    customerId: record.customerId,
    expiresAt: record.expiresAt,
    items: record.items.map((i) => ({
      productId: i.productId,
      quantity: i.quantity,
    })),
  };
}
