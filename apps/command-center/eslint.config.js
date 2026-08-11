import tsParser from "@typescript-eslint/parser";
import ts from "@typescript-eslint/eslint-plugin";
import solid from "eslint-plugin-solid";

export default [
  { ignores: [".output/**", ".vinxi/**", "node_modules/**", "dist/**"] },
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: { ecmaVersion: "latest", sourceType: "module" },
    },
    plugins: { "@typescript-eslint": ts, solid },
    rules: {
      ...ts.configs.recommended.rules,
      ...solid.configs.typescript.rules,
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
];
