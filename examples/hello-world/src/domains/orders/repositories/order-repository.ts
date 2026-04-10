import type { PrismaClient } from "@prisma/client";
import type { Order, OrderStatus } from "../types";

export async function createOrder(
  db: PrismaClient,
  data: {
    customerId: string;
    totalInCents: number;
    lines: {
      productId: string;
      productName: string;
      priceInCents: number;
      quantity: number;
    }[];
  },
): Promise<Order> {
  const record = await db.order.create({
    data: {
      customerId: data.customerId,
      totalInCents: data.totalInCents,
      status: "pending",
      lines: { create: data.lines },
    },
    include: { lines: true },
  });

  return toOrder(record);
}

export async function findOrderById(
  db: PrismaClient,
  id: string,
): Promise<Order | null> {
  const record = await db.order.findUnique({
    where: { id },
    include: { lines: true },
  });
  return record ? toOrder(record) : null;
}

export async function listOrdersByCustomer(
  db: PrismaClient,
  customerId: string,
): Promise<Order[]> {
  const records = await db.order.findMany({
    where: { customerId },
    include: { lines: true },
    orderBy: { createdAt: "desc" },
  });
  return records.map(toOrder);
}

export async function updateOrderStatus(
  db: PrismaClient,
  id: string,
  status: OrderStatus,
): Promise<void> {
  await db.order.update({ where: { id }, data: { status } });
}

function toOrder(record: {
  id: string;
  customerId: string;
  status: string;
  totalInCents: number;
  createdAt: Date;
  lines: {
    productId: string;
    productName: string;
    priceInCents: number;
    quantity: number;
  }[];
}): Order {
  return {
    id: record.id,
    customerId: record.customerId,
    status: record.status as OrderStatus,
    totalInCents: record.totalInCents,
    createdAt: record.createdAt,
    lines: record.lines.map((l) => ({
      productId: l.productId,
      productName: l.productName,
      priceInCents: l.priceInCents,
      quantity: l.quantity,
    })),
  };
}
