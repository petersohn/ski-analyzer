import { execSync } from "child_process";
import { existsSync, mkdirSync, readdirSync, rmSync } from "fs";
import { join, basename } from "path";

const SCHEMAS_DIR = "src-tauri/ski-analyzer-lib/target";
const OUTPUT_TYPES_DIR = "src/app/types/generated";

function findSchemas(): string[] {
  const patterns = [
    `${SCHEMAS_DIR}/debug/out/schemas`,
    `${SCHEMAS_DIR}/release/out/schemas`,
  ];

  for (const pattern of patterns) {
    if (existsSync(pattern)) {
      return readdirSync(pattern)
        .filter((f) => f.endsWith(".json"))
        .map((f) => join(pattern, f));
    }
  }
  return [];
}

function ensureDir(dir: string): void {
  rmSync(dir, { force: true, recursive: true });
  mkdirSync(dir, { recursive: true });
}

function generateTypes(schemas: string[]): void {
  ensureDir(OUTPUT_TYPES_DIR);

  for (const schema of schemas) {
    const baseName = basename(schema, ".json");
    const tsName = baseName.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
    const outputFile = join(OUTPUT_TYPES_DIR, `${tsName}.ts`);

    try {
      execSync(
        `quicktype "${schema}" -o "${outputFile}" --top-level ${tsName} --src-lang schema --just-types`,
        {
          stdio: "inherit",
        },
      );
      console.log(`Generated: ${tsName}.ts`);
    } catch (e) {
      console.error(`Failed to generate types for ${schema}`);
    }
  }
}

function main(): void {
  console.log("Finding schemas...");
  const schemas = findSchemas();

  if (schemas.length === 0) {
    console.log(
      "No schemas found. Run: cd src-tauri/ski-analyzer-lib && cargo build --features schemars",
    );
    return;
  }

  console.log(`Found ${schemas.length} schemas`);

  console.log("\nGenerating TypeScript types...");
  generateTypes(schemas);

  console.log("\nDone!");
}

main();
