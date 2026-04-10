import type { PrismaClient } from "@prisma/client";
import type { Product } from "../types";
import { ProductNotFoundError, InsufficientInventoryError } from "../errors";
import {
  listProducts,
  findProductById,
  decrementStock,
  incrementStock,
} from "../repositories/product-repository";

export async function getProducts(
  db: PrismaClient,
  options?: { cursor?: string; take?: number },
): Promise<Product[]> {
  return listProducts(db, options);
}

export async function getProductById(
  db: PrismaClient,
  id: string,
): Promise<Product> {
  const product = await findProductById(db, id);
  if (!product) throw new ProductNotFoundError(id);
  return product;
}

export async function reserveStock(
  db: PrismaClient,
  productId: string,
  quantity: number,
): Promise<Product> {
  const product = await findProductById(db, productId);
  if (!product) throw new ProductNotFoundError(productId);
  if (product.stockCount < quantity) {
    throw new InsufficientInventoryError(productId, quantity, product.stockCount);
  }
  await decrementStock(db, productId, quantity);
  return { ...product, stockCount: product.stockCount - quantity };
}

export async function releaseStock(
  db: PrismaClient,
  productId: string,
  quantity: number,
): Promise<void> {
  await incrementStock(db, productId, quantity);
}
