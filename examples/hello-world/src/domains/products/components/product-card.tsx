import type { Product } from "../types";

export function ProductCard({ product }: { product: Product }) {
  const price = (product.priceInCents / 100).toFixed(2);
  const inStock = product.stockCount > 0;

  return (
    <div className="rounded-lg border border-gray-200 p-4">
      <h3 className="font-semibold">{product.name}</h3>
      <p className="mt-1 text-sm text-gray-600">{product.description}</p>
      <div className="mt-3 flex items-center justify-between">
        <span className="font-mono text-sm">{price} EUR</span>
        <span
          className={`text-xs font-medium ${inStock ? "text-green-600" : "text-red-500"}`}
        >
          {inStock ? `${product.stockCount} in stock` : "Out of stock"}
        </span>
      </div>
    </div>
  );
}
