const native = require('./index.js');
const util = require('util');

function patchInspector(Klass, methodName = 'toString', forcePlainObject = false) {
  if (!Klass || typeof Klass !== 'function') {
    console.log('inspect not a function')
    return;
  }
  const proto = Klass.prototype;
  const original = proto[util.inspect.custom];
  proto[util.inspect.custom] = function(...args) {
    if (typeof this[methodName] === 'function') {
      const result = this[methodName](...args);
      if (forcePlainObject && result && typeof result === 'object') {
        return { ...result };
      }
      return result;
    }
    if (typeof original === 'function') {
      return original.apply(this, args);
    }
    return { ...this };
  };
}

function wrapNativeFunction(fn) {
  if (typeof fn !== 'function') return fn;
  return function(...args) {
    try {
      const result = fn.apply(this, args);
      if (result instanceof Promise) {
        return result.catch(error => {
          throw mapNativeError(error);
        });
      }
      return result;
    } catch (error) {
      throw mapNativeError(error);
    }
  };
}

function wrapClassMethods(Class) {
  const prototype = Class.prototype;
  const methods = Object.getOwnPropertyNames(prototype);
  methods.forEach(method => {
    if (method !== 'constructor' && typeof prototype[method] === 'function') {
      prototype[method] = wrapNativeFunction(prototype[method]);
    }
  });
  return Class;
}

function wrapClass(Class, inspectOptions) {
  const Wrapped = wrapClassMethods(Class);
  patchInspector(Wrapped, ...(inspectOptions || []));
  return Wrapped;
}

// Custom error classes
class ElementNotFoundError extends Error {
    constructor(message) {
        super(message);
        this.name = 'ElementNotFoundError';
    }
}

class TimeoutError extends Error {
    constructor(message) {
        super(message);
        this.name = 'TimeoutError';
    }
}

class PermissionDeniedError extends Error {
    constructor(message) {
        super(message);
        this.name = 'PermissionDeniedError';
    }
}

class PlatformError extends Error {
    constructor(message) {
        super(message);
        this.name = 'PlatformError';
    }
}

class UnsupportedOperationError extends Error {
    constructor(message) {
        super(message);
        this.name = 'UnsupportedOperationError';
    }
}

class UnsupportedPlatformError extends Error {
    constructor(message) {
        super(message);
        this.name = 'UnsupportedPlatformError';
    }
}

class InvalidArgumentError extends Error {
    constructor(message) {
        super(message);
        this.name = 'InvalidArgumentError';
    }
}

class InternalError extends Error {
    constructor(message) {
        super(message);
        this.name = 'InternalError';
    }
}

// Error mapping function
function mapNativeError(error) {
    if (!error.message) return error;

    const message = error.message;
    if (message.startsWith('ELEMENT_NOT_FOUND:')) {
        return new ElementNotFoundError(message.replace('ELEMENT_NOT_FOUND:', '').trim());
    }
    if (message.startsWith('OPERATION_TIMED_OUT:')) {
        return new TimeoutError(message.replace('OPERATION_TIMED_OUT:', '').trim());
    }
    if (message.startsWith('PERMISSION_DENIED:')) {
        return new PermissionDeniedError(message.replace('PERMISSION_DENIED:', '').trim());
    }
    if (message.startsWith('PLATFORM_ERROR:')) {
        return new PlatformError(message.replace('PLATFORM_ERROR:', '').trim());
    }
    if (message.startsWith('UNSUPPORTED_OPERATION:')) {
        return new UnsupportedOperationError(message.replace('UNSUPPORTED_OPERATION:', '').trim());
    }
    if (message.startsWith('UNSUPPORTED_PLATFORM:')) {
        return new UnsupportedPlatformError(message.replace('UNSUPPORTED_PLATFORM:', '').trim());
    }
    if (message.startsWith('INVALID_ARGUMENT:')) {
        return new InvalidArgumentError(message.replace('INVALID_ARGUMENT:', '').trim());
    }
    if (message.startsWith('INTERNAL_ERROR:')) {
        return new InternalError(message.replace('INTERNAL_ERROR:', '').trim());
    }
    return error;
}

// Enhanced executeBrowserScript with function and file support
async function enhancedExecuteBrowserScript(scriptOrFunction, envOrOptions) {
  const fs = require('fs');
  const path = require('path');

  let script;
  let env = {};

  // Handle different input types
  if (typeof scriptOrFunction === 'string') {
    // Check if it's a file path
    if (scriptOrFunction.endsWith('.ts') || scriptOrFunction.endsWith('.js')) {
      // File path - read and compile
      const filePath = path.resolve(scriptOrFunction);
      if (!fs.existsSync(filePath)) {
        throw new Error(`Browser script file not found: ${filePath}`);
      }

      let fileContent = fs.readFileSync(filePath, 'utf-8');

      // If TypeScript, compile it
      if (filePath.endsWith('.ts')) {
        try {
          const esbuild = require('esbuild');
          const result = await esbuild.transform(fileContent, {
            loader: 'ts',
            target: 'es2020',
            format: 'iife'
          });
          fileContent = result.code;
        } catch (e) {
          // If esbuild not available, try to use as-is (may work for simple TS)
          console.warn('esbuild not found - using TypeScript file as-is:', e.message);
        }
      }

      script = fileContent;
      env = envOrOptions || {};
    } else {
      // Plain string script - use as-is (backward compatible)
      script = scriptOrFunction;
    }
  } else if (typeof scriptOrFunction === 'function') {
    // Function - convert to IIFE with proper wrapping
    const funcString = scriptOrFunction.toString();
    env = envOrOptions || {};

    // Wrap function in IIFE that handles return values
    script = `
      (async function() {
        const fn = ${funcString};
        const result = await fn(${JSON.stringify(env)});

        // Auto-stringify result if it's an object
        if (result !== undefined && result !== null) {
          if (typeof result === 'object') {
            return JSON.stringify(result);
          }
          return String(result);
        }
        return null;
      })()
    `;
  } else if (typeof scriptOrFunction === 'object' && scriptOrFunction.file) {
    // Object with file property
    const filePath = path.resolve(scriptOrFunction.file);
    if (!fs.existsSync(filePath)) {
      throw new Error(`Browser script file not found: ${filePath}`);
    }

    let fileContent = fs.readFileSync(filePath, 'utf-8');

    // If TypeScript, compile it
    if (filePath.endsWith('.ts')) {
      try {
        const esbuild = require('esbuild');
        const result = await esbuild.transform(fileContent, {
          loader: 'ts',
          target: 'es2020',
          format: 'iife'
        });
        fileContent = result.code;
      } catch (e) {
        console.warn('esbuild not found - using TypeScript file as-is:', e.message);
      }
    }

    script = fileContent;
    env = scriptOrFunction.env || {};
  } else {
    throw new Error('Invalid argument to executeBrowserScript: expected string, function, or {file, env} object');
  }

  // Call the original native method
  const resultStr = await this._originalExecuteBrowserScript(script);

  // If function was passed, try to parse JSON result
  if (typeof scriptOrFunction === 'function') {
    try {
      return JSON.parse(resultStr);
    } catch (e) {
      // Not JSON, return as-is
      return resultStr;
    }
  }

  // For string/file, return raw result (backward compatible)
  return resultStr;
}

// Wrap the native classes
const Desktop = wrapClassMethods(native.Desktop);
const Element = wrapClass(native.Element);
const Locator = wrapClass(native.Locator);
const Selector = wrapClass(native.Selector);

// Patch executeBrowserScript on Desktop and Element
if (Desktop.prototype.executeBrowserScript) {
  Desktop.prototype._originalExecuteBrowserScript = Desktop.prototype.executeBrowserScript;
  Desktop.prototype.executeBrowserScript = enhancedExecuteBrowserScript;
}

if (Element.prototype.executeBrowserScript) {
  Element.prototype._originalExecuteBrowserScript = Element.prototype.executeBrowserScript;
  Element.prototype.executeBrowserScript = enhancedExecuteBrowserScript;
}

// Export everything
module.exports = {
    Desktop,
    Element,
    Locator,
    Selector,
    // Export error classes
    ElementNotFoundError,
    TimeoutError,
    PermissionDeniedError,
    PlatformError,
    UnsupportedOperationError,
    UnsupportedPlatformError,
    InvalidArgumentError,
    InternalError
};
