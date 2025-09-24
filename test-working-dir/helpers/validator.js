// Helper module for testing relative imports
module.exports = {
    validate: function(value) {
        console.log(`[validator.js] Validating value: ${value}`);
        return value !== null && value !== undefined;
    },

    getMessage: function() {
        return "Helper module loaded successfully!";
    }
};