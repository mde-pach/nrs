import { PrismaClient } from "@prisma/client";
import { execSync } from "child_process";
import path from "path";
import fs from "fs";

const PRISMA_DIR = path.join(__dirname, "../../prisma");
const TEST_DB_PATH = path.join(PRISMA_DIR, "test.db");
const TEST_DB_URL = `file:${TEST_DB_PATH}`;

let _db: PrismaClient | null = null;
let _initialized = false;

export function getTestDb(): PrismaClient {
  if (!_initialized) {
    execSync("npx prisma db push --skip-generate --accept-data-loss", {
      env: { ...process.env, DATABASE_URL: TEST_DB_URL },
      cwd: path.join(__dirname, "../.."),
      stdio: "ignore",
    });
    _initialized = true;
  }

  if (!_db) {
    _db = new PrismaClient({
      datasources: { db: { url: TEST_DB_URL } },
    });
  }

  return _db;
}

export async function cleanTestDb(db: PrismaClient): Promise<void> {
  await db.orderLine.deleteMany();
  await db.order.deleteMany();
  await db.cartItem.deleteMany();
  await db.cart.deleteMany();
  await db.product.deleteMany();
  await db.category.deleteMany();
}

export async function destroyTestDb(): Promise<void> {
  if (_db) {
    await _db.$disconnect();
    _db = null;
  }
  _initialized = false;
  if (fs.existsSync(TEST_DB_PATH)) fs.unlinkSync(TEST_DB_PATH);
  const journal = `${TEST_DB_PATH}-journal`;
  if (fs.existsSync(journal)) fs.unlinkSync(journal);
}
