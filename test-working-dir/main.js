// Main script that uses relative imports
const validator = require('./helpers/validator.js');

console.log('[main.js] Script started');
console.log('[main.js] Testing relative import...');

const message = validator.getMessage();
console.log(`[main.js] ${message}`);

const testValue = "test";
const isValid = validator.validate(testValue);
console.log(`[main.js] Validation result: ${isValid}`);

// Return success result
return {
    status: "success",
    message: "Script executed with relative imports working correctly",
    validatorMessage: message,
    workingDirectory: process.cwd()
};