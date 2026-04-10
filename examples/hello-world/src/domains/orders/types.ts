export type OrderStatus =
  | "pending"
  | "confirmed"
  | "shipped"
  | "delivered"
  | "cancelled"
  | "refunded";

export type OrderLine = {
  productId: string;
  productName: string;
  priceInCents: number;
  quantity: number;
};

export type Order = {
  id: string;
  customerId: string;
  lines: OrderLine[];
  status: OrderStatus;
  totalInCents: number;
  createdAt: Date;
};

export type Cart = {
  id: string;
  customerId: string;
  items: CartItem[];
  expiresAt: Date;
};

export type CartItem = {
  productId: string;
  quantity: number;
};
