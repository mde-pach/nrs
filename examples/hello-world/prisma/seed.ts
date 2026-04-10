import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient();

async function main() {
  const electronics = await prisma.category.create({
    data: { name: "Electronics" },
  });

  const clothing = await prisma.category.create({
    data: { name: "Clothing" },
  });

  await prisma.product.createMany({
    data: [
      {
        name: "Wireless Headphones",
        description: "Noise-cancelling over-ear headphones",
        priceInCents: 9999,
        stockCount: 50,
        categoryId: electronics.id,
      },
      {
        name: "USB-C Cable",
        description: "2m braided charging cable",
        priceInCents: 1299,
        stockCount: 200,
        categoryId: electronics.id,
      },
      {
        name: "Mechanical Keyboard",
        description: "Cherry MX Brown switches, tenkeyless",
        priceInCents: 14999,
        stockCount: 30,
        categoryId: electronics.id,
      },
      {
        name: "Cotton T-Shirt",
        description: "Plain black, 100% organic cotton",
        priceInCents: 2499,
        stockCount: 100,
        categoryId: clothing.id,
      },
      {
        name: "Denim Jacket",
        description: "Classic fit, medium wash",
        priceInCents: 7999,
        stockCount: 0,
        categoryId: clothing.id,
      },
    ],
  });

  console.log("Seeded database");
}

main()
  .then(() => prisma.$disconnect())
  .catch((e) => {
    console.error(e);
    prisma.$disconnect();
    process.exit(1);
  });
