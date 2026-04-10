export class OrderNotFoundError extends Error {
  constructor(orderId: string) {
    super(`Order not found: ${orderId}`);
    this.name = "OrderNotFoundError";
  }
}

export class OrderNotCancellableError extends Error {
  constructor(orderId: string, status: string) {
    super(`Order ${orderId} cannot be cancelled in status: ${status}`);
    this.name = "OrderNotCancellableError";
  }
}

export class EmptyCartError extends Error {
  constructor(customerId: string) {
    super(`Cart is empty for customer: ${customerId}`);
    this.name = "EmptyCartError";
  }
}
