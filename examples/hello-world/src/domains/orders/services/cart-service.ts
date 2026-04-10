import type { PrismaClient } from "@prisma/client";
import type { Cart } from "../types";
import {
  findCartByCustomer,
  upsertCart,
  deleteCart,
} from "../repositories/cart-repository";

export async function getCart(
  db: PrismaClient,
  customerId: string,
): Promise<Cart | null> {
  return findCartByCustomer(db, customerId);
}

export async function setCartItems(
  db: PrismaClient,
  customerId: string,
  items: { productId: string; quantity: number }[],
): Promise<Cart> {
  return upsertCart(db, customerId, items);
}

export async function clearCart(
  db: PrismaClient,
  customerId: string,
): Promise<void> {
  await deleteCart(db, customerId);
}
