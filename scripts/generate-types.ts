import { execSync } from "child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
  unlinkSync,
} from "fs";
import { join } from "path";

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
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
}

function cleanDir(dir: string): void {
  if (existsSync(dir)) {
    for (const file of readdirSync(dir)) {
      unlinkSync(join(dir, file));
    }
  }
}

function generateAllTypes(schemas: string[]): string {
  const tmpFile = "/tmp/all-types.ts";
  const schemaArgs = schemas.map((s) => `"${s}"`).join(" ");

  execSync(
    `npx quicktype --src-lang schema --lang ts --just-types --out "${tmpFile}" ${schemaArgs}`,
    { stdio: "pipe" },
  );

  return readFileSync(tmpFile, "utf-8");
}

interface TypeInfo {
  kind: "interface" | "enum" | "type";
  definition: string;
}

function extractTypeDefs(content: string): Map<string, TypeInfo> {
  const types = new Map<string, TypeInfo>();

  // Extract interfaces with proper brace matching
  let pos = 0;
  while (pos < content.length) {
    const interfaceStart = content.indexOf("export interface ", pos);
    const enumStart = content.indexOf("export enum ", pos);
    const typeStart = content.indexOf("type ", pos);

    let start = -1;
    let kind: "interface" | "enum" | "type" = "interface";

    if (
      interfaceStart !== -1 &&
      (enumStart === -1 || interfaceStart < enumStart) &&
      (typeStart === -1 || interfaceStart < typeStart)
    ) {
      start = interfaceStart;
      kind = "interface";
    } else if (
      enumStart !== -1 &&
      (typeStart === -1 || enumStart < typeStart)
    ) {
      start = enumStart;
      kind = "enum";
    } else if (typeStart !== -1) {
      // Check it's a top-level type alias (not inside a definition)
      const lineStart = content.lastIndexOf("\n", typeStart) + 1;
      if (content.substring(lineStart, typeStart).trim() === "") {
        start = typeStart;
        kind = "type";
      }
    }

    if (start === -1) break;

    if (kind === "interface") {
      const nameStart = start + "export interface ".length;
      const braceStart = content.indexOf("{", nameStart);
      if (braceStart === -1) break;

      const name = content
        .substring(nameStart, braceStart)
        .trim()
        .split(/[<\s]/)[0];

      // Find matching closing brace
      let depth = 0;
      let i = braceStart;
      while (i < content.length) {
        if (content[i] === "{") depth++;
        if (content[i] === "}") {
          depth--;
          if (depth === 0) {
            types.set(name, {
              kind: "interface",
              definition: content.substring(start, i + 1),
            });
            pos = i + 1;
            break;
          }
        }
        i++;
      }
      if (depth !== 0) break;
    } else if (kind === "enum") {
      const nameStart = start + "export enum ".length;
      const braceStart = content.indexOf("{", nameStart);
      if (braceStart === -1) break;

      const name = content.substring(nameStart, braceStart).trim();
      const braceEnd = content.indexOf("}", braceStart);
      if (braceEnd === -1) break;

      types.set(name, {
        kind: "enum",
        definition: content.substring(start, braceEnd + 1),
      });
      pos = braceEnd + 1;
    } else {
      // type alias
      const lineEnd = content.indexOf(";", start);
      if (lineEnd === -1) break;

      const line = content.substring(start, lineEnd + 1);
      const nameMatch = line.match(/^type (\w+)\s*=/);
      if (nameMatch) {
        types.set(nameMatch[1], { kind: "type", definition: line });
      }
      pos = lineEnd + 1;
    }
  }

  return types;
}

function getCanonicalFile(typeName: string): string | null {
  const typeToFiles: Record<string, string> = {
    Point: "point",
    Rect: "rect",
    LineString: "lineString",
    MultiLineString: "multiLineString",
    Polygon: "polygon",
    MultiPolygon: "multiPolygon",
    OffsetDateTime: "offsetDateTime",
    PointWithElevation: "pointWithElevation",
    BoundedLineString: "boundedLineString",
    BoundedPolygon: "boundedPolygon",
    Lift: "lift",
    Difficulty: "difficulty",
    PisteMetadata: "pisteMetadata",
    PisteData: "pisteData",
    Piste: "piste",
    SkiAreaMetadata: "skiAreaMetadata",
    SkiArea: "skiArea",
  };
  return typeToFiles[typeName] || null;
}

function splitIntoFiles(allTypes: Map<string, TypeInfo>): Map<string, string> {
  const files = new Map<string, string>();

  // Group types by their canonical file
  const typesByFile = new Map<string, string[]>();

  for (const [typeName, { definition }] of allTypes) {
    const canonicalFile = getCanonicalFile(typeName);
    if (canonicalFile) {
      const existing = typesByFile.get(canonicalFile) || [];
      existing.push(definition);
      typesByFile.set(canonicalFile, existing);
    }
  }

  // Build each file
  for (const [fileName, definitions] of typesByFile) {
    files.set(fileName, definitions.join("\n\n") + "\n");
  }

  return files;
}

function addImports(files: Map<string, string>): void {
  for (const [fileName, content] of files) {
    const imports = new Set<string>();

    // Find all type references
    const patterns = [
      /:\s*(\w+)/g, // Type annotations
      /(\w+)\[\]/g, // Array types
      /<(\w+)>/g, // Generic types
      /{\s*\[key:\s*string\]:\s*(\w+)/g, // Record types
    ];

    for (const pattern of patterns) {
      const matches = content.matchAll(pattern);
      for (const match of matches) {
        const typeName = match[1];
        const canonicalFile = getCanonicalFile(typeName);
        if (canonicalFile && canonicalFile !== fileName) {
          imports.add(typeName);
        }
      }
    }

    // Build import statements
    const importsByFile = new Map<string, string[]>();
    for (const typeName of imports) {
      const sourceFile = getCanonicalFile(typeName);
      if (sourceFile) {
        const existing = importsByFile.get(sourceFile) || [];
        existing.push(typeName);
        importsByFile.set(sourceFile, existing);
      }
    }

    let importStatements = "";
    for (const [sourceFile, typeNames] of importsByFile) {
      importStatements += `import { ${typeNames.join(", ")} } from "./${sourceFile}";\n`;
    }

    if (importStatements) {
      files.set(fileName, importStatements + "\n" + content);
    }
  }
}

function writeFiles(files: Map<string, string>): void {
  for (const [fileName, content] of files) {
    writeFileSync(join(OUTPUT_TYPES_DIR, `${fileName}.ts`), content);
    console.log(`Generated: ${fileName}.ts`);
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
  ensureDir(OUTPUT_TYPES_DIR);
  cleanDir(OUTPUT_TYPES_DIR);

  const allContent = generateAllTypes(schemas);
  const allTypes = extractTypeDefs(allContent);
  const files = splitIntoFiles(allTypes);
  addImports(files);
  writeFiles(files);

  console.log("\nDone!");
}

main();
