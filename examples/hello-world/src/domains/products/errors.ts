export class ProductNotFoundError extends Error {
  constructor(productId: string) {
    super(`Product not found: ${productId}`);
    this.name = "ProductNotFoundError";
  }
}

export class InsufficientInventoryError extends Error {
  constructor(productId: string, requested: number, available: number) {
    super(
      `Insufficient inventory for ${productId}: requested ${requested}, available ${available}`,
    );
    this.name = "InsufficientInventoryError";
  }
}
