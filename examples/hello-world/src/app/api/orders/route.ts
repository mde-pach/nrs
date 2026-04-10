import { NextResponse } from "next/server";
import { prisma } from "@/shared/prisma";
import { getOrdersByCustomer } from "@/domains/orders/services/order-service";
import { checkout } from "@/domains/orders/services/checkout-service";
import { EmptyCartError } from "@/domains/orders/errors";
import { InsufficientInventoryError } from "@/domains/products/errors";

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const customerId = searchParams.get("customerId");

  if (!customerId) {
    return NextResponse.json(
      { error: "customerId is required" },
      { status: 400 },
    );
  }

  const orders = await getOrdersByCustomer(prisma, customerId);
  return NextResponse.json({ orders });
}

export async function POST(request: Request) {
  const body = await request.json();
  const { customerId } = body;

  if (!customerId) {
    return NextResponse.json(
      { error: "customerId is required" },
      { status: 400 },
    );
  }

  try {
    const order = await checkout(prisma, customerId);
    return NextResponse.json({ order }, { status: 201 });
  } catch (error) {
    if (error instanceof EmptyCartError) {
      return NextResponse.json({ error: error.message }, { status: 400 });
    }
    if (error instanceof InsufficientInventoryError) {
      return NextResponse.json({ error: error.message }, { status: 409 });
    }
    throw error;
  }
}
