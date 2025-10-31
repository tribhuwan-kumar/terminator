/**
 * Example TypeScript browser script
 * This file can be executed directly with executeBrowserScript('./example-script.ts')
 */

interface PageInfo {
  title: string;
  url: string;
  linkCount: number;
  imageCount: number;
  hasLoginForm: boolean;
}

// IIFE that returns JSON string
(function(): string {
  const pageInfo: PageInfo = {
    title: document.title,
    url: window.location.href,
    linkCount: document.querySelectorAll('a').length,
    imageCount: document.querySelectorAll('img').length,
    hasLoginForm: !!(
      document.querySelector('input[type="password"]') ||
      document.querySelector('input[name*="pass"]')
    )
  };

  return JSON.stringify(pageInfo);
})();
