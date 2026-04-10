import { NextResponse } from "next/server";
import { prisma } from "@/shared/prisma";
import {
  getCart,
  setCartItems,
  clearCart,
} from "@/domains/orders/services/cart-service";

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const customerId = searchParams.get("customerId");

  if (!customerId) {
    return NextResponse.json(
      { error: "customerId is required" },
      { status: 400 },
    );
  }

  const cart = await getCart(prisma, customerId);
  return NextResponse.json({ cart });
}

export async function PUT(request: Request) {
  const body = await request.json();
  const { customerId, items } = body;

  if (!customerId || !items) {
    return NextResponse.json(
      { error: "customerId and items are required" },
      { status: 400 },
    );
  }

  const cart = await setCartItems(prisma, customerId, items);
  return NextResponse.json({ cart });
}

export async function DELETE(request: Request) {
  const { searchParams } = new URL(request.url);
  const customerId = searchParams.get("customerId");

  if (!customerId) {
    return NextResponse.json(
      { error: "customerId is required" },
      { status: 400 },
    );
  }

  await clearCart(prisma, customerId);
  return NextResponse.json({ ok: true });
}
