import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "fs";
import { join } from "path";

const SCHEMAS_DIR = "src-tauri/ski-analyzer-lib/target";
const OUTPUT_TYPES_DIR = "src/app/types/generated";

interface JsonSchema {
  type?: string;
  properties?: Record<string, JsonSchema>;
  required?: string[];
  enum?: string[];
  items?: JsonSchema;
  $ref?: string;
  additionalProperties?: JsonSchema;
  format?: string;
  minimum?: number;
}

interface SchemaFile {
  [typeName: string]: JsonSchema;
}

const TYPE_NAME_MAP: Record<string, string> = {
  PointDef: "Point",
  RectDef: "Rect",
  LineStringDef: "LineString",
  MultiLineStringDef: "MultiLineString",
  PolygonDef: "Polygon",
  MultiPolygonDef: "MultiPolygon",
  OffsetDateTimeDef: "string",
  BoundedGeometryDef_for_PolygonDef: "BoundedGeometry<Polygon>",
  BoundedGeometryDef_for_LineStringDef: "BoundedGeometry<LineString>",
};

function findSchema(): string | null {
  const patterns = [
    `${SCHEMAS_DIR}/debug/out/schemas/ski-analyzer.json`,
    `${SCHEMAS_DIR}/release/out/schemas/ski-analyzer.json`,
  ];

  for (const pattern of patterns) {
    if (existsSync(pattern)) {
      return pattern;
    }
  }
  return null;
}

function ensureDir(dir: string): void {
  rmSync(dir, { force: true, recursive: true });
  mkdirSync(dir, { recursive: true });
}

function resolveRef(ref: string): string {
  const parts = ref.split("/");
  const typeName = parts[parts.length - 1];
  return TYPE_NAME_MAP[typeName] ?? typeName;
}

function schemaTypeToTs(schema: JsonSchema): string {
  if (schema.$ref) {
    return resolveRef(schema.$ref);
  }

  if (schema.enum) {
    return schema.enum.map((v) => JSON.stringify(v)).join(" | ");
  }

  switch (schema.type) {
    case "string":
      return "string";
    case "number":
    case "integer":
      return "number";
    case "boolean":
      return "boolean";
    case "array":
      if (schema.items) {
        const itemType = schemaTypeToTs(schema.items);
        return `${itemType}[]`;
      }
      return "any[]";
    case "object":
      if (schema.additionalProperties) {
        const valueType = schemaTypeToTs(schema.additionalProperties);
        return `Record<string, ${valueType}>`;
      }
      if (schema.properties) {
        return generateInlineInterface(schema);
      }
      return "Record<string, any>";
    default:
      return "any";
  }
}

function generateInlineInterface(schema: JsonSchema): string {
  if (!schema.properties) {
    return "Record<string, any>";
  }

  const fields = Object.entries(schema.properties).map(([key, value]) => {
    const tsType = schemaTypeToTs(value);
    return `${key}: ${tsType}`;
  });

  return `{ ${fields.join("; ")} }`;
}

function generateInterface(name: string, schema: JsonSchema): string {
  const lines: string[] = [];

  if (schema.enum) {
    const members = schema.enum
      .map((v) => `  | ${JSON.stringify(v)}`)
      .join("\n");
    lines.push(`export type ${name} =`);
    lines.push(`${members};`);
    return lines.join("\n");
  }

  if (schema.type === "string" && !schema.properties) {
    lines.push(`export type ${name} = string;`);
    return lines.join("\n");
  }

  if (schema.type === "array" && schema.items) {
    const itemType = schemaTypeToTs(schema.items);
    lines.push(`export type ${name} = ${itemType}[];`);
    return lines.join("\n");
  }

  if (!schema.properties) {
    lines.push(`export type ${name} = any;`);
    return lines.join("\n");
  }

  lines.push(`export type ${name} = {`);

  for (const [propName, propSchema] of Object.entries(schema.properties)) {
    const tsType = schemaTypeToTs(propSchema);
    lines.push(`  ${propName}: ${tsType};`);
  }

  lines.push("};");
  return lines.join("\n");
}

const GEO_TYPE_NAMES = new Set([
  "PointDef",
  "RectDef",
  "LineStringDef",
  "MultiLineStringDef",
  "PolygonDef",
  "MultiPolygonDef",
]);

function isGeoRef(typeName: string): boolean {
  return GEO_TYPE_NAMES.has(typeName);
}

const GEO_TYPES = [
  "PointDef",
  "RectDef",
  "LineStringDef",
  "MultiLineStringDef",
  "PolygonDef",
  "MultiPolygonDef",
];

const SKI_AREA_TYPES = [
  "PointWithElevation",
  "Difficulty",
  "Lift",
  "PisteMetadata",
  "PisteData",
  "Piste",
  "SkiAreaMetadata",
  "SkiArea",
];

function collectDirectRefs(schema: JsonSchema): string[] {
  const refs: string[] = [];

  function collect(s: JsonSchema) {
    if (s.$ref) {
      const parts = s.$ref.split("/");
      refs.push(parts[parts.length - 1]);
    }
    if (s.properties) {
      Object.values(s.properties).forEach(collect);
    }
    if (s.items) {
      collect(s.items);
    }
    if (s.additionalProperties) {
      collect(s.additionalProperties);
    }
  }

  collect(schema);
  return [...new Set(refs)];
}

function generateGeoFile(schema: SchemaFile): void {
  const outputLines: string[] = [];

  for (const typeName of GEO_TYPES) {
    const schemaDef = schema[typeName];
    if (!schemaDef) {
      console.warn(`Type ${typeName} not found in schema`);
      continue;
    }

    const exportName = TYPE_NAME_MAP[typeName] ?? typeName;
    outputLines.push(generateInterface(exportName, schemaDef));
    outputLines.push("");
  }

  outputLines.push(`export type BoundedGeometry<T> = {`);
  outputLines.push(`  item: T;`);
  outputLines.push(`  bounding_rect: Rect;`);
  outputLines.push(`};`);

  const outputFile = join(OUTPUT_TYPES_DIR, "geo.ts");
  writeFileSync(outputFile, outputLines.join("\n") + "\n");
  console.log("Generated: geo.ts");
}

function generateSkiAreaFile(schema: SchemaFile): void {
  const outputLines: string[] = [];

  // Collect geo imports
  const geoImports = new Set<string>();
  for (const typeName of SKI_AREA_TYPES) {
    const schemaDef = schema[typeName];
    if (!schemaDef) continue;
    const refs = collectDirectRefs(schemaDef);
    for (const ref of refs) {
      if (isGeoRef(ref)) {
        const mapped = TYPE_NAME_MAP[ref] ?? ref;
        if (mapped !== "string") {
          geoImports.add(mapped);
        }
      }
    }
  }

  // Check if BoundedGeometry is needed and add its type params to imports
  for (const typeName of SKI_AREA_TYPES) {
    const schemaDef = schema[typeName];
    if (!schemaDef) continue;
    const refs = collectDirectRefs(schemaDef);
    for (const ref of refs) {
      if (ref.startsWith("BoundedGeometryDef_")) {
        geoImports.add("BoundedGeometry");
        const mapped = TYPE_NAME_MAP[ref] ?? ref;
        // Extract type parameter from BoundedGeometry<LineString> etc.
        const match = mapped.match(/BoundedGeometry<(\w+)>/);
        if (match) {
          geoImports.add(match[1]);
        }
      }
    }
  }

  if (geoImports.size > 0) {
    outputLines.push(`import { ${[...geoImports].join(", ")} } from "./geo";`);
    outputLines.push("");
  }

  for (const typeName of SKI_AREA_TYPES) {
    const schemaDef = schema[typeName];
    if (!schemaDef) {
      console.warn(`Type ${typeName} not found in schema`);
      continue;
    }

    const exportName = TYPE_NAME_MAP[typeName] ?? typeName;
    outputLines.push(generateInterface(exportName, schemaDef));
    outputLines.push("");
  }

  const outputFile = join(OUTPUT_TYPES_DIR, "skiArea.ts");
  writeFileSync(outputFile, outputLines.join("\n"));
  console.log("Generated: skiArea.ts");
}

function main(): void {
  console.log("Finding schema...");
  const schemaPath = findSchema();

  if (!schemaPath) {
    console.log(
      "No schema found. Run: cd src-tauri/ski-analyzer-lib && cargo build --features schemars",
    );
    return;
  }

  console.log(`Found schema: ${schemaPath}`);

  const schemaContent = readFileSync(schemaPath, "utf-8");
  const schema: SchemaFile = JSON.parse(schemaContent);

  console.log(`Schema contains ${Object.keys(schema).length} types`);

  ensureDir(OUTPUT_TYPES_DIR);

  console.log("\nGenerating TypeScript types...");
  generateGeoFile(schema);
  generateSkiAreaFile(schema);

  console.log("\nDone!");
}

main();
