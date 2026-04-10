import type { PrismaClient } from "@prisma/client";
import type { Order } from "../types";
import { EmptyCartError } from "../errors";
import { findCartByCustomer, deleteCart } from "../repositories/cart-repository";
import { createOrder } from "../repositories/order-repository";
import {
  getProductById,
  reserveStock,
} from "../../products/services/product-service";

export async function checkout(
  db: PrismaClient,
  customerId: string,
): Promise<Order> {
  return db.$transaction(async (tx) => {
    const cart = await findCartByCustomer(tx as PrismaClient, customerId);
    if (!cart || cart.items.length === 0) {
      throw new EmptyCartError(customerId);
    }

    const lines: {
      productId: string;
      productName: string;
      priceInCents: number;
      quantity: number;
    }[] = [];

    for (const item of cart.items) {
      const product = await getProductById(tx as PrismaClient, item.productId);
      await reserveStock(tx as PrismaClient, item.productId, item.quantity);

      lines.push({
        productId: product.id,
        productName: product.name,
        priceInCents: product.priceInCents,
        quantity: item.quantity,
      });
    }

    const totalInCents = lines.reduce(
      (sum, l) => sum + l.priceInCents * l.quantity,
      0,
    );

    const order = await createOrder(tx as PrismaClient, {
      customerId,
      totalInCents,
      lines,
    });

    await deleteCart(tx as PrismaClient, customerId);

    return order;
  });
}
