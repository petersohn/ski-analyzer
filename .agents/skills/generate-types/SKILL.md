---
name: generate-types
description: Describes how to write types that are used for the communication between the library and the frontend. Use when dealing with any type that derives from specta::Type or when creating new types that are used as members in types that derive from specta::Type. Also consult this skill before modifying anything in src/app/types.
---

## Summary

This skill describes the procedure of generating Typescript types from rust that are used in the frontend. The main idea is that we shouldn't duplicate types because that would run the risk of misalignment of communication. The basic rules are the following.
- Any type that is used in both TypeScript and Rust must have its TypeScript version generated from the Rust code.
- The type generation code must be under the `specta` feature flag.
- For any Rust type that's defined within out codebase, the specta derive should be added to the type.
- For Rust types defined outside the codebase, shadow types should be created.

## How to annotate types?

When creating a type that requires a TypeScript representation, follow the following steps.

**Step 1: Annotate the type with the necessary derive.**

```rust
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MyType {
    pub foo: Foo,
    ...
}
```

**Step 2: Add the type to `typescript_generator.ts`.**

```rust
    let types = specta::TypeCollection::default()
        ...
        .register::<MyType>()
        ...
```

**Step 3: Regenerate types.**

Generate the TypeScript types with this command.

```sh
pnpm generate-types
```

**Important:** Don't invoke `typescript_generator` directly, because it will create unneeded output files. The above command should verify that type generation is correct.

### Handling generic types

When registering generic types, a generic argument for the type is needed. However, the generated TypeScript type is still generic, so it's enough to register only one of the type instances. For example:

```rust
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MyType<T> {
    ...
}

// Later this is used in different ways.
let foo: MyType<Foo> = ...;
let bar: MyType<Bar> = ...;
```

Then in `typescript_generator.ts`, register any of the instances, but not all of them. For example:

```rust
        .register::<MyType<Foo>>()
```

### Handling typed from external libraries

If a type is defined outside our codebase, then we cannot add a specta annotation. Instead, we need to create shadow types.

**Important**: Only use shadow types if absolutely necessary. If a type is defined within our codebase, use the above described method to add annotations.

**Step 1: Create the shadow type**

In `src-tauri/ski-analyzer-lib/src/typescript_gen/<library_name>.rs`, create the shadow type. Use `serde(rename)` to generate the correct name for the type. For example, the following shadow type of `geo::Point` is in `geo.rs`.

```rust
#[derive(Serialize, Type)]
#[serde(rename = "Point")]
pub struct PointDef {
    pub x: f64,
    pub y: f64,
}
```

**Step 2: Use the shadow type in our type definition**

```rust
#[cfg(feature = "specta")]
use crate::typescript_gen::geo::PointDef;

...

#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MyType {
    #[cfg_attr(feature = "specta", specta(type = PointDef))]
    pub point: Point,
    ...
}
```

## Using generated TypeScript types

By default, a generated type can be used as-is from TypeScript. For clarity, the types are categorized into different Typescript modules. For example, `MyType` belongs to `Foo` category, in which case it should be re-exported from `src/app/types/foo.ts`.

```typescript
export { MyType } from "./generated/generated";
```

### Converting data

Sometimes, it is inconvenient to use the deserialized JSON directly from the frontend. In this case, conversion is needed. The most common reason is that maps are deserialized as TypeScript objects, but it's better to treat them as `Map`s instead. There are other reasons conversion might be necessary, but we are using maps as an example.

For example, we have this Rust type:

```rust
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MyType {
    pub foo: Foo,
    pub bars: HashMap<String, Bar>,
}
```

Then it should be converted like this:

```typescript
import { indexData } from "@/utils/data";
import { MyType as RawMyType } from "./generated/generated";

export { Foo, Bar } from "./generated/generated";

export { RawMyType };

export type MyType {
    foo: Foo;
    bars: Map<string, Bar>;
}

export function indexMyType(my_type: RawMyType): MyType {
    return {
        foo: my_type.foo,
        bar: indexData<Bar>(my_type.bars),
    };
}
```

#### Converting data recursively

If a type needs to be converted, then any type which has a member of a convertable type needs to be converted too. For example, if `Foo` and `Bar` also need to be converted, then the above code will look like this.

```typescript
import { indexAndConvertData } from "@/utils/data";
import { MyType as RawMyType, Foo as RawFoo, Bar as RawBar } from "./generated/generated";

// RawFoo and RawBar only need to be exported if they are used as a root message.
export { RawMyType };

export type Foo {
    ...
}

export type Bar {
    ...
}

export type MyType {
    foo: Foo;
    bars: Map<string, Bar>;
}

// Conversion functions only need to be exported if they are used as a root message.
function indexFoo(foo: RawFoo): Foo {
    ...
}

function indexBar(bar: RawBar): Bar {
    ...
}

export function indexMyType(my_type: RawMyType): MyType {
    return {
        foo: indexFoo(my_type.foo),
        bars: indexAndConvertData<RawBar, Bar>(my_type.bars, indexBar),
    };
}
```

## When to regenerate types?

Call `pnpm generate-types` when:
- `src/app/types/generated/generated.ts` does not exist.
- `typescript_generator.ts` is modified. For example, because a new type that needs a TypeScript representation is added or removed.
- The definition of a type with a `specta::Type` derive is modified.
- The TypeScript compiler reports an error where the cause is that `generated.ts` is not up to date.

**Important:** Don't modify `generated.ts` directly. If a modification is needed, modify on the Rust side or do a conversion.
