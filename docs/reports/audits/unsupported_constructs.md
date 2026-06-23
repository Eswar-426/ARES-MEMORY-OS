# Unsupported Constructs

During the validation sprint, we identified structural patterns that ARES gracefully skips over, but does not yet deeply model.

## Medium Priority
- **Complex TypeScript Decorators:** Basic structural info is retained for NestJS, but deep decorator parameter parsing (e.g., extracting HTTP route paths from `@Get('/path')`) is not fully mapped to requirement edges yet.
- **Rust Macros:** `macro_rules!` and advanced procedural macros are treated as opaque nodes. The internal logic is not decomposed.
- **Dynamic Imports:** JavaScript `await import(...)` strings constructed dynamically are not resolved into hard edges.

## Low Priority
- **CSS / SASS Modules:** Currently ignored. No requirement edges are drawn to style sheets.
- **Monorepo Task Pipelines:** Turborepo `turbo.json` and Nx `project.json` task dependencies are not modeled as graph edges yet, only the source code is.

*Note: None of these caused failures or crashes. They simply represent areas where the graph could be richer.*
