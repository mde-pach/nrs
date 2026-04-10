# Server Components

## Rendering Strategy

Server Components by default. Client Components only when the component requires browser interactivity (event handlers, hooks, browser APIs).

## Client/Server Wrapper Pattern

When a component needs interactivity but most of its content could be server-rendered, split it into two parts:

- A **Server Component** that fetches data, handles authorization, and renders the structural layout
- A **Client Component** that receives only the data it needs as props and handles the interactive parts

The server component imports and renders the client component, passing serializable props. This keeps the data fetching and heavy rendering on the server, while the client component is as small as possible.

```tsx
// Server Component — does the data fetching
async function ProductPage({ id }: { id: string }) {
  const product = await getProductById(prisma, id);
  return (
    <article>
      <h1>{product.name}</h1>
      <p>{product.description}</p>
      <AddToCartButton productId={product.id} price={product.priceInCents} />
    </article>
  );
}

// Client Component — only the interactive part
"use client";
function AddToCartButton({ productId, price }: { productId: string; price: number }) {
  const [adding, setAdding] = useState(false);
  // ... interaction logic
}
```

## Rules

- Never add `"use client"` to a component just because it imports a client component — the parent can stay as a server component
- Data fetching (database, API calls) happens only in server components
- Client components receive pre-fetched data as serializable props — they never call the database directly
- When in doubt, start as a server component. Convert to client only when you hit a compilation error requiring it.
