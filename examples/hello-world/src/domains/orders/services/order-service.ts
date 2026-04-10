import type { PrismaClient } from "@prisma/client";
import type { Order } from "../types";
import { OrderNotFoundError, OrderNotCancellableError } from "../errors";
import {
  findOrderById,
  listOrdersByCustomer,
  updateOrderStatus,
} from "../repositories/order-repository";
import { releaseStock } from "../../products/services/product-service";

export async function getOrder(
  db: PrismaClient,
  id: string,
): Promise<Order> {
  const order = await findOrderById(db, id);
  if (!order) throw new OrderNotFoundError(id);
  return order;
}

export async function getOrdersByCustomer(
  db: PrismaClient,
  customerId: string,
): Promise<Order[]> {
  return listOrdersByCustomer(db, customerId);
}

export async function cancelOrder(
  db: PrismaClient,
  id: string,
): Promise<void> {
  const order = await findOrderById(db, id);
  if (!order) throw new OrderNotFoundError(id);
  if (order.status !== "pending") {
    throw new OrderNotCancellableError(id, order.status);
  }

  await db.$transaction(async (tx) => {
    await updateOrderStatus(tx as PrismaClient, id, "cancelled");
    for (const line of order.lines) {
      await releaseStock(tx as PrismaClient, line.productId, line.quantity);
    }
  });
}
