// Fixture: invalid — each line below should report `no-double-cast`.

type DbRow = { id: string; data: unknown };
type Dto = { id: string; name: string };

export function bad1(row: DbRow): Dto {
  // expect: report — classic "as unknown as T" tunnel.
  return row as unknown as Dto;
}

export function bad2(row: DbRow): Dto {
  // expect: report — parenthesized cast still nests TSAsExpression.
  return (row as unknown) as Dto;
}

export function bad3(input: string): number {
  // expect: report — chain doesn't have to go through `unknown`.
  return input as any as number;
}
