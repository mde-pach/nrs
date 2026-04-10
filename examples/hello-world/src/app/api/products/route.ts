import { NextResponse } from "next/server";
import { prisma } from "@/shared/prisma";
import { getProducts } from "@/domains/products/services/product-service";

export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const cursor = searchParams.get("cursor") ?? undefined;
  const take = Number(searchParams.get("take")) || 20;

  const products = await getProducts(prisma, { cursor, take });
  return NextResponse.json({ products });
}
