// Fixture: valid — single casts are fine; nothing should fire.

type Dto = { id: string };

export function good1(x: unknown): Dto {
  // Single cast — necessary boundary, type system can still see it.
  return x as Dto;
}

export function good2(x: number): string {
  // Single cast through `as`.
  return String(x) as string;
}

// Two SEPARATE single casts on the same line — also fine, no nesting.
export function good3(a: unknown, b: unknown): [Dto, Dto] {
  return [a as Dto, b as Dto];
}
