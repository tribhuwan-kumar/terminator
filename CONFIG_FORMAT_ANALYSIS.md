# Configuration Format Analysis for Terminator Workflows

## The Question

What format should we use for workflow configuration?
- Pure config (YAML/TOML/JSON)?
- Extend existing files (package.json)?
- TypeScript config (like next.config.ts)?
- New format?

## What is "terminator.yml" Actually?

Looking at `workflows/imperial_treasure_1/terminator.yaml`, it's **NOT just config** - it's:

1. **Workflow Definition** - Steps, order, logic
2. **Variables** - Input schema, defaults
3. **Metadata** - Name, description, version, tags
4. **Runtime Behavior** - Conditions, error handling, branching

This is **more than config** - it's a **workflow spec**.

## Modern Tools Analysis

### Vercel's Approach

```json
// vercel.json - deployment config only
{
  "builds": [...],
  "routes": [...],
  "env": {...}
}
```

```typescript
// next.config.ts - framework config with logic
export default {
  async redirects() {
    return [...]; // Can execute code!
  },
  webpack: (config) => {
    config.plugins.push(...);
    return config;
  }
}
```

**Insight:** Vercel separates:
- **vercel.json** = Pure deployment config
- **next.config.ts** = Framework config with JS logic

### Mastra.ai's Approach

```typescript
// mastra.config.ts - pure TypeScript
export default defineConfig({
  workflows: {
    'my-workflow': './workflows/my-workflow.ts'
  },
  llm: { ... },
  tools: { ... }
});
```

**Insight:** Everything is TypeScript - no YAML/JSON at all!

### Inngest's Approach

```typescript
// No separate config file
// Workflows ARE the config
export default inngest.createFunction(
  { id: 'my-function' },
  { event: 'user.signup' },
  async ({ event }) => { ... }
);
```

**Insight:** Code IS config. Discovery via file scanning.

### Temporal's Approach

```yaml
# temporal.yaml - server config only
server:
  port: 7233

# Workflows are pure code
```

```typescript
// workflow.ts
export async function myWorkflow() { ... }
```

**Insight:** Config is for infrastructure, workflows are code.

### ESLint's Evolution

```javascript
// OLD: .eslintrc.json (JSON)
{
  "rules": { ... }
}

// NEW: eslint.config.js (JS with logic)
export default [
  {
    rules: {
      'no-console': process.env.NODE_ENV === 'production' ? 'error' : 'warn'
    }
  }
];
```

**Insight:** Industry moving from static config to executable config.

### Vite's Approach

```typescript
// vite.config.ts
export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: { ... }
  }
});
```

**Insight:** TypeScript config with type safety and autocomplete.

## Format Comparison

### YAML
```yaml
# Pros
- Human readable
- Comments
- Multi-line strings
- Industry standard for config

# Cons
- No type safety
- Syntax errors common (indentation)
- No logic/computation
- Requires parser
- Security (YAML bombs)
```

**Good for:** Static configuration, CI/CD, Kubernetes
**Bad for:** Workflows with logic, type safety

### TOML
```toml
# Pros
- Very readable
- Simple syntax
- Good for nested config
- Comments

# Cons
- Less common than YAML/JSON
- No type safety
- No logic
- Smaller ecosystem
```

**Good for:** App config (Cargo.toml, pyproject.toml)
**Bad for:** Workflows, complex schemas

### JSON
```json
// Pros
{
  "fast": "to parse",
  "universal": "every language",
  "safe": "no code execution"
}

// Cons
{
  "no_comments": "major pain point",
  "no_trailing_commas": "annoying",
  "verbose": "lots of quotes",
  "no_logic": "static only"
}
```

**Good for:** Data exchange, package.json, APIs
**Bad for:** Human editing, complex config

### TypeScript Config
```typescript
// Pros
export default {
  typeSafety: '✅ Full IntelliSense',
  logic: () => 'Can execute code',
  validation: 'Compile-time checks',
  imports: 'Can import other files',
  functions: (x) => x * 2,
};

// Cons
const cons = {
  needsRuntime: 'Must execute to read',
  buildStep: 'Might need compilation',
  uiParsing: 'Harder for non-JS tools',
};
```

**Good for:** Framework config, complex logic, type safety
**Bad for:** Pure data, non-JS tools

## Our Options

### Option 1: Pure YAML (Traditional)

```yaml
# workflow.yml - Everything in YAML
id: my-workflow
name: My Workflow
version: 1.0.0

variables:
  userName:
    type: string
    default: World

steps:
  - id: step1
    function: doSomething
    args: [...]
```

```typescript
// workflow.ts - Just code
export async function doSomething(desktop: Desktop) {
  // Implementation
}
```

**Pros:**
- ✅ Clear separation (config vs code)
- ✅ Easy for UI to parse
- ✅ Human readable
- ✅ No build step

**Cons:**
- ❌ No type safety for config
- ❌ YAML syntax errors
- ❌ Can't compute values
- ❌ Two files to maintain

### Option 2: Extend package.json (Vercel-style)

```json
// package.json
{
  "name": "my-workflow",
  "version": "1.0.0",
  "terminator": {
    "id": "my-workflow",
    "name": "My Workflow",
    "variables": {
      "userName": {
        "type": "string",
        "default": "World"
      }
    },
    "steps": [...]
  }
}
```

**Pros:**
- ✅ Single file (package.json)
- ✅ Standard npm metadata
- ✅ Version in one place

**Cons:**
- ❌ No comments in JSON
- ❌ Gets crowded
- ❌ Not for non-npm projects

### Option 3: TypeScript Config (Vite-style)

```typescript
// terminator.config.ts
export default defineConfig({
  id: 'my-workflow',
  name: 'My Workflow',
  version: '1.0.0',

  variables: {
    userName: {
      type: 'string',
      default: process.env.DEFAULT_USER || 'World', // Can compute!
    },
  },

  steps: [
    { id: 'step1', function: 'doSomething' },
  ],
});
```

**Pros:**
- ✅ Type safety
- ✅ Can compute values
- ✅ IntelliSense/autocomplete
- ✅ Single source of truth

**Cons:**
- ❌ Needs execution to read
- ❌ Harder for UI to parse (need to run it)

### Option 4: TypeScript is Config (Mastra/Inngest-style)

```typescript
// workflow.ts - Code IS config
import { createWorkflow } from '@mediar/terminator';

export default createWorkflow({
  id: 'my-workflow',
  name: 'My Workflow',
  version: '1.0.0',
})
  .variable('userName', { type: 'string', default: 'World' })
  .step('step1', async ({ desktop }) => {
    // Implementation directly here
  });
```

**Pros:**
- ✅ Single file
- ✅ Type safety
- ✅ No sync issues
- ✅ Modern DX

**Cons:**
- ❌ Harder for UI to parse
- ❌ Needs execution

### Option 5: Hybrid (Our Current Proposal)

```yaml
# workflow.yml - Metadata + Structure
id: my-workflow
name: My Workflow
steps:
  - id: step1
    function: doSomething
```

```typescript
// workflow.ts - Implementation
export async function doSomething(desktop: Desktop) {
  // Code
}
```

**Pros:**
- ✅ Best of both worlds
- ✅ UI can parse YAML easily
- ✅ Code has type safety
- ✅ Clear separation

**Cons:**
- ❌ Two files to maintain
- ❌ Can get out of sync

## What Does mediar-app Need?

The UI needs to:

1. **List workflows** - Scan for workflow files
2. **Display metadata** - Name, description, version
3. **Render form** - Variable inputs
4. **Show steps** - Step names, descriptions
5. **Execute** - Run the workflow

### If YAML:
```typescript
// UI can parse directly
const yaml = await fs.readFile('workflow.yml');
const config = parseYAML(yaml);

// Display in UI
config.steps.map(step => <StepCard {...step} />);

// Execute
await executeWorkflow(config);
```

### If TypeScript Config:
```typescript
// UI needs to execute the config
const config = await import('./workflow.ts');
const resolved = await config.default;

// Display in UI
resolved.steps.map(step => <StepCard {...step} />);
```

**Challenge:** UI runs in browser, can't easily execute Node.js code.

**Solution:** Build step that compiles to JSON?

## Recommended Approach

### For Simple Workflows: YAML + TS (Option 5)

Keep it simple:
```
workflow/
├── workflow.yml      # Metadata (UI reads this)
├── workflow.ts       # Implementation
└── package.json      # Dependencies
```

**Why:**
- ✅ UI can parse YAML directly (no execution needed)
- ✅ TypeScript for code (type safety)
- ✅ Clear separation of concerns
- ✅ Works for non-npm projects too

### For Advanced Workflows: TypeScript Config with Compilation

```
workflow/
├── terminator.config.ts    # Full TypeScript config
├── terminator.config.json  # Compiled output (UI reads this)
├── package.json
└── tsconfig.json
```

**Workflow:**
1. Developer writes `terminator.config.ts` (type-safe)
2. CLI compiles to `terminator.config.json` (for UI)
3. UI reads compiled JSON
4. Runtime executes TypeScript

```typescript
// terminator.config.ts
export default defineConfig({
  id: 'my-workflow',
  name: 'My Workflow',
  version: packageJson.version, // Can reference!

  variables: {
    environment: {
      type: 'string',
      default: process.env.NODE_ENV || 'production', // Can compute!
    },
  },

  steps: [
    { id: 'step1', name: 'Do Thing', function: 'doThing' },
  ],
});
```

**Build command:**
```bash
terminator build  # Compiles .ts → .json
```

**Output:**
```json
{
  "id": "my-workflow",
  "name": "My Workflow",
  "version": "1.0.0",
  "variables": {
    "environment": {
      "type": "string",
      "default": "production"
    }
  },
  "steps": [...]
}
```

## Industry Trend: TypeScript Config

Modern tools are moving to TypeScript config:

- ✅ **ESLint** - `.eslintrc.json` → `eslint.config.js`
- ✅ **Vite** - Uses `vite.config.ts`
- ✅ **Next.js** - Uses `next.config.ts`
- ✅ **Tailwind** - `tailwind.config.ts`
- ✅ **Vitest** - `vitest.config.ts`
- ✅ **Drizzle ORM** - `drizzle.config.ts`

**Why?** Because config needs logic!

## Final Recommendation

### Phase 1 (MVP): YAML + TypeScript

```yaml
# workflow.yml
id: my-workflow
name: My Workflow
steps:
  - id: step1
    function: doSomething
```

```typescript
// workflow.ts
export function doSomething(desktop: Desktop) { }
```

**Rationale:**
- Simple to implement
- UI can parse YAML directly
- No build step required
- Clear separation

### Phase 2 (Advanced): TypeScript Config with Build

```typescript
// terminator.config.ts
export default defineConfig({
  id: 'my-workflow',
  name: 'My Workflow',
  version: pkg.version,
  variables: {
    env: {
      default: process.env.NODE_ENV
    }
  }
});
```

```bash
$ terminator build
✓ Compiled terminator.config.ts → terminator.config.json
```

**Rationale:**
- Type safety
- Can compute values
- Follows industry trend
- UI still reads static JSON

### Phase 3 (Future): Code is Config

```typescript
// workflow.ts - Everything in one file
export default createWorkflow({
  id: 'my-workflow',
  name: 'My Workflow',
})
  .step('step1', async ({ desktop }) => {
    // Implementation
  });
```

**Rationale:**
- Best DX
- Single source of truth
- Full flexibility

## Comparison Table

| Format | Type Safety | Logic | UI Parsing | Comments | Build Step | Ecosystem |
|--------|-------------|-------|------------|----------|------------|-----------|
| YAML | ❌ | ❌ | ✅ Easy | ✅ | ❌ No | ✅ Large |
| TOML | ❌ | ❌ | ⚠️ Medium | ✅ | ❌ No | ⚠️ Medium |
| JSON | ❌ | ❌ | ✅ Easy | ❌ | ❌ No | ✅ Large |
| package.json | ❌ | ❌ | ✅ Easy | ❌ | ❌ No | ✅ Standard |
| TS Config | ✅ | ✅ | ⚠️ Needs build | ✅ | ✅ Yes | ✅ Growing |
| Code as Config | ✅ | ✅ | ❌ Hard | ✅ | ⚠️ Maybe | ⚠️ Small |

## Answer to Your Question

**"What should be our approach?"**

### Start Here (Like Vercel):

```
workflow/
├── workflow.yml        # Metadata (like vercel.json)
├── workflow.ts         # Code (like Next.js pages)
└── package.json        # Dependencies
```

### Evolve To (Like Next.js):

```
workflow/
├── terminator.config.ts    # Config with logic (like next.config.ts)
├── terminator.config.json  # Compiled (for UI)
├── workflow.ts             # Implementation
└── package.json
```

### End Goal (Like Mastra):

```
workflow/
├── workflow.ts         # Everything (code is config)
└── package.json
```

**Why this path?**

1. **Phase 1 is simple** - Get it working fast
2. **Phase 2 adds power** - Type safety when needed
3. **Phase 3 is optional** - For power users

Just like how Vercel has both `vercel.json` (simple) and complex Next.js apps (advanced), we should support both simple YAML workflows and advanced TypeScript workflows.

## Don't Overthink It

The best config format is the one that matches your user:

- **Beginners** → YAML (readable)
- **Developers** → TypeScript (type-safe)
- **Production** → Compiled JSON (fast, safe)

Support all three. Start with YAML. Add TypeScript later. Everyone wins. ✅
