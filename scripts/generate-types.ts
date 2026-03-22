import { execSync } from "child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  writeFileSync,
  unlinkSync,
} from "fs";
import { join, basename } from "path";

const SCHEMAS_DIR = "src-tauri/ski-analyzer-lib/target";
const OUTPUT_TYPES_DIR = "src/app/types/generated";

// Explicit mappings for types that quicktype names differently
const TYPE_RENAMES: Record<string, string> = {
  LiftValue: "Lift",
  PisteValue: "Piste",
  Metadata: "SkiAreaMetadata",
  BoundingRect: "Rect",
  Bound: "Rect",
  Max: "Point",
  MaxElement: "Point",
  Line: "BoundedLineString",
  Outline: "BoundedPolygon",
  Item: "Polygon",
  AreaElement: "Polygon",
  StationElement: "PointWithElevation",
};

// Canonical file for each type (where the type should be defined)
// Maps type name -> file name (without .ts)
const TYPE_HOME: Record<string, string> = {
  Point: "point",
  Rect: "rect",
  Lift: "lift",
  Piste: "piste",
  SkiAreaMetadata: "skiAreaMetadata",
  SkiArea: "skiArea",
  Difficulty: "difficulty",
  PointWithElevation: "pointWithElevation",
  BoundedLineString: "boundedLineString",
  BoundedPolygon: "boundedPolygon",
  Polygon: "polygon",
  LineString: "lineString",
  MultiLineString: "multiLineString",
  MultiPolygon: "multiPolygon",
  OffsetDateTime: "offsetDateTime",
  PisteData: "pisteData",
  PisteMetadata: "pisteMetadata",
  BoundedGeometryLineString: "boundedGeometryLineString",
  BoundedGeometryPolygon: "boundedGeometryPolygon",
};

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

function generateTypes(schemas: string[]): Map<string, string> {
  ensureDir(OUTPUT_TYPES_DIR);
  cleanDir(OUTPUT_TYPES_DIR);

  const generatedFiles = new Map<string, string>();

  for (const schema of schemas) {
    const baseName = basename(schema, ".json");
    const tsName = baseName.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
    const outputFile = join(OUTPUT_TYPES_DIR, `${tsName}.ts`);

    try {
      execSync(
        `quicktype "${schema}" -o "${outputFile}" --top-level ${tsName} --src-lang schema --just-types`,
        { stdio: "pipe" },
      );
      generatedFiles.set(tsName, outputFile);
      console.log(`Generated: ${tsName}.ts`);
    } catch (e) {
      console.error(`Failed to generate types for ${schema}`);
    }
  }

  return generatedFiles;
}

function extractTypeDefs(
  content: string,
): Array<{ name: string; fullMatch: string; start: number; end: number }> {
  const types: Array<{
    name: string;
    fullMatch: string;
    start: number;
    end: number;
  }> = [];

  // Extract interfaces with proper brace matching
  let pos = 0;
  while (pos < content.length) {
    const interfaceStart = content.indexOf("export interface ", pos);
    const enumStart = content.indexOf("export enum ", pos);

    let start = -1;
    let isInterface = false;

    if (
      interfaceStart !== -1 &&
      (enumStart === -1 || interfaceStart < enumStart)
    ) {
      start = interfaceStart;
      isInterface = true;
    } else if (enumStart !== -1) {
      start = enumStart;
      isInterface = false;
    }

    if (start === -1) break;

    if (isInterface) {
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
            types.push({
              name,
              fullMatch: content.substring(start, i + 1),
              start,
              end: i + 1,
            });
            pos = i + 1;
            break;
          }
        }
        i++;
      }
      if (depth !== 0) break;
    } else {
      // enum
      const nameStart = start + "export enum ".length;
      const braceStart = content.indexOf("{", nameStart);
      if (braceStart === -1) break;

      const name = content.substring(nameStart, braceStart).trim();
      const braceEnd = content.indexOf("}", braceStart);
      if (braceEnd === -1) break;

      types.push({
        name,
        fullMatch: content.substring(start, braceEnd + 1),
        start,
        end: braceEnd + 1,
      });
      pos = braceEnd + 1;
    }
  }

  return types;
}

function postProcessTypes(files: Map<string, string>): void {
  console.log("\nPost-processing to remove duplicates...");

  // First pass: rename types in all files
  for (const [tsName, filePath] of files) {
    let content = readFileSync(filePath, "utf-8");

    // Rename types
    for (const [oldName, newName] of Object.entries(TYPE_RENAMES)) {
      // Rename definitions
      content = content.replace(
        new RegExp(`(export (?:interface|enum|type)) ${oldName}\\b`, "g"),
        `$1 ${newName}`,
      );
      // Rename references
      content = content.replace(
        new RegExp(`:\\s*${oldName}\\b`, "g"),
        `: ${newName}`,
      );
      content = content.replace(
        new RegExp(`<${oldName}\\b`, "g"),
        `<${newName}`,
      );
      content = content.replace(
        new RegExp(`\\b${oldName}\\[\\]`, "g"),
        `${newName}[]`,
      );
    }

    writeFileSync(filePath, content);
  }

  // Second pass: remove duplicates and add imports
  for (const [tsName, filePath] of files) {
    let content = readFileSync(filePath, "utf-8");
    const types = extractTypeDefs(content);

    const importsByFile = new Map<string, Set<string>>();
    const typesToRemove: string[] = [];

    for (const type of types) {
      const homeFile = TYPE_HOME[type.name];

      // If this type belongs to a different file, remove it and import
      if (homeFile && homeFile !== tsName) {
        typesToRemove.push(type.fullMatch);

        if (!importsByFile.has(homeFile)) {
          importsByFile.set(homeFile, new Set());
        }
        importsByFile.get(homeFile)!.add(type.name);
      }
    }

    // Remove duplicate type definitions
    for (const toRemove of typesToRemove) {
      content = content.replace(toRemove + "\n", "");
      content = content.replace(toRemove, "");
    }

    // Build import statements
    let imports = "";
    for (const [sourceFile, typeNames] of importsByFile) {
      imports += `import { ${[...typeNames].join(", ")} } from "./${sourceFile}";\n`;
    }

    // Combine and clean up
    content = imports + content;
    content = content.replace(/\n{3,}/g, "\n\n").trim() + "\n";

    writeFileSync(filePath, content);
  }

  console.log("Post-processing complete.");
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
  const generatedFiles = generateTypes(schemas);

  postProcessTypes(generatedFiles);

  console.log("\nDone!");
}

main();
