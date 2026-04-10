import { prisma } from "@/shared/prisma";
import { getProducts } from "@/domains/products/services/product-service";
import { ProductCard } from "@/domains/products/components/product-card";

export default async function Home() {
  const products = await getProducts(prisma);

  return (
    <main className="mx-auto max-w-4xl p-8">
      <h1 className="text-2xl font-bold">Hello World Commerce</h1>
      <p className="mt-2 text-gray-600">
        NRS example project — see the *.context.md files for how context is
        organized.
      </p>

      <section className="mt-8">
        <h2 className="text-lg font-semibold">Products</h2>
        <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {products.map((product) => (
            <ProductCard key={product.id} product={product} />
          ))}
        </div>
        {products.length === 0 && (
          <p className="mt-4 text-gray-500">
            No products yet. Run <code>npm run db:setup</code> to seed the
            database.
          </p>
        )}
      </section>
    </main>
  );
}
