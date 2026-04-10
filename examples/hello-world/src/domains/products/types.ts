export type Product = {
  id: string;
  name: string;
  description: string;
  priceInCents: number;
  stockCount: number;
  categoryId: string;
  archived: boolean;
};

export type Category = {
  id: string;
  name: string;
  parentId: string | null;
};
