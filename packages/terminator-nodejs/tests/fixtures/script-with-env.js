// Browser script that uses injected env variables
(() => {
  // column_positions should be auto-injected by wrapper
  if (typeof column_positions === 'undefined') {
    throw new Error('column_positions not injected');
  }

  return {
    positions: column_positions,
    count: column_positions.length
  };
})()
