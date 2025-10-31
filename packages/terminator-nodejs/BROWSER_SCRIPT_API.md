# Enhanced executeBrowserScript API

## Overview

The `executeBrowserScript` method now supports 3 ways to execute JavaScript in browser contexts:

1. **Plain string** (backward compatible)
2. **Function** (new - auto IIFE wrapping, auto JSON handling)
3. **TypeScript/JavaScript file** (new - auto compilation)

## Why This Matters

**Before:**
```typescript
const resultStr = await chromeWindow.executeBrowserScript(`
  (function() {
    const data = document.querySelector('.data');
    return JSON.stringify({ found: true, text: data.textContent });
  })()
`);
const result = JSON.parse(resultStr); // Manual parsing
```

**After:**
```typescript
const result = await chromeWindow.executeBrowserScript(() => {
  const data = document.querySelector('.data');
  return { found: true, text: data.textContent }; // Auto JSON handling!
});
// result is already an object!
```

## API Reference

### Method Signature

```typescript
executeBrowserScript(
  scriptOrFunction: string | Function | { file: string; env?: any },
  envOrOptions?: any
): Promise<string | any>
```

### Usage Patterns

#### Pattern 1: Plain String (Backward Compatible)

```typescript
const result = await element.executeBrowserScript(`
  (function() {
    return JSON.stringify({ title: document.title });
  })()
`);
const parsed = JSON.parse(result); // Manual parsing required
```

**Returns:** `string` (raw result)

#### Pattern 2: Function (Recommended for Simple Scripts)

```typescript
// No IIFE wrapping needed!
const result = await element.executeBrowserScript(() => {
  return {
    title: document.title,
    links: document.querySelectorAll('a').length
  };
});

// result is already parsed: { title: '...', links: 10 }
```

**Returns:** `any` (auto JSON.parse if valid JSON)

**With environment variables:**
```typescript
const result = await element.executeBrowserScript((env) => {
  const selector = env.selector;
  const elements = document.querySelectorAll(selector);

  return {
    selector: selector,
    count: elements.length
  };
}, { selector: '.item' });
```

#### Pattern 3: TypeScript File Path

**Create a file: `scripts/extract-data.ts`**
```typescript
interface DataResult {
  items: string[];
  count: number;
}

(function(): string {
  const elements = document.querySelectorAll('.item');
  const result: DataResult = {
    items: Array.from(elements).map(el => el.textContent),
    count: elements.length
  };

  return JSON.stringify(result);
})();
```

**Execute it:**
```typescript
const resultStr = await element.executeBrowserScript('./scripts/extract-data.ts');
const result = JSON.parse(resultStr);
```

**Returns:** `string` (raw result, file content executed as-is)

#### Pattern 4: File with Environment Variables

```typescript
const result = await element.executeBrowserScript({
  file: './scripts/extract-data.ts',
  env: { selector: '.custom-item', maxResults: 50 }
});
```

**Note:** For files, env variables must be accessed within the file script (not automatically injected yet).

## TypeScript Compilation

When you pass a `.ts` file path, the wrapper automatically:

1. Reads the file
2. Compiles it with `esbuild` (if available)
3. Executes the compiled JavaScript

**Requirements:**
- `esbuild` must be installed (`npm install esbuild`)
- If esbuild is not available, the TS file is used as-is (may work for simple TypeScript)

## Complete Example

```typescript
import { Desktop } from '@mediar-ai/terminator';

const desktop = new Desktop();
const chromeWindow = await desktop.locator('role:Window|name:Chrome').first(5000);

// Example 1: Function with full TypeScript support
const pageInfo = await chromeWindow.executeBrowserScript(() => {
  return {
    title: document.title,
    url: window.location.href,
    meta: {
      description: document.querySelector('meta[name="description"]')?.content,
      keywords: document.querySelector('meta[name="keywords"]')?.content
    },
    stats: {
      links: document.querySelectorAll('a').length,
      images: document.querySelectorAll('img').length,
      forms: document.querySelectorAll('form').length
    }
  };
});

console.log('Page:', pageInfo.title);
console.log('Links:', pageInfo.stats.links);

// Example 2: Reusable TypeScript script file
const formData = await chromeWindow.executeBrowserScript({
  file: './browser-scripts/extract-form.ts',
  env: { formId: 'login-form' }
});
```

## Migration Guide

### Old Way (Still Works!)
```typescript
const result = await window.executeBrowserScript(`
  (function() {
    const title = document.title;
    return JSON.stringify({ title: title });
  })()
`);
const parsed = JSON.parse(result);
```

### New Way (Cleaner!)
```typescript
const result = await window.executeBrowserScript(() => {
  return { title: document.title };
});
// No JSON.parse needed!
```

## Benefits

✅ **No more IIFE boilerplate** - just write the function body
✅ **Auto JSON handling** - return objects directly, get objects back
✅ **TypeScript support** - write browser scripts in `.ts` files with full type safety
✅ **Reusable scripts** - extract complex browser logic into separate files
✅ **IDE support** - syntax highlighting, autocomplete, type checking for browser scripts
✅ **Backward compatible** - existing string-based scripts still work

## Limitations

1. Functions are stringified and re-parsed in browser context - closures don't work
2. Environment variables for functions are JSON-serialized (objects only, no functions)
3. TypeScript compilation requires `esbuild` dependency

## Advanced: Async Operations

Functions are automatically wrapped in `async` context, so Promises work:

```typescript
const result = await window.executeBrowserScript(async () => {
  // Wait for element to appear
  await new Promise(resolve => {
    const check = setInterval(() => {
      if (document.querySelector('.loaded')) {
        clearInterval(check);
        resolve();
      }
    }, 100);
  });

  return {
    loaded: true,
    content: document.querySelector('.content').textContent
  };
});
```

## See Also

- [example-browser-script.js](./example-browser-script.js) - Complete working example
- [example-script.ts](./example-script.ts) - TypeScript browser script example
