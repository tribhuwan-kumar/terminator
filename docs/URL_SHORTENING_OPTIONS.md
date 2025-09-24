# URL Shortening Options for Terminator CLI Installation

## Current URLs (Long)
- macOS/Linux: `https://raw.githubusercontent.com/mediar-ai/terminator/main/scripts/install.sh`
- Windows: `https://raw.githubusercontent.com/mediar-ai/terminator/main/scripts/install.ps1`

## Option 1: Domain Redirect (RECOMMENDED)
Set up redirects on mediar.ai domain:

### Setup:
1. Add these redirects to your web server (nginx/Apache/Cloudflare):
   - `mediar.ai/install.sh` → GitHub raw install.sh
   - `mediar.ai/install.ps1` → GitHub raw install.ps1
   - `mediar.ai/install` → Landing page with platform detection

### Usage:
```bash
# macOS/Linux
curl -fsSL https://mediar.ai/install.sh | bash

# Windows
irm https://mediar.ai/install.ps1 | iex

# Or even shorter with custom subdomain
curl -fsSL https://get.mediar.ai | bash
```

### Cloudflare Setup Example:
```
# Page Rules or Workers
mediar.ai/install.sh → 301 Redirect → https://raw.githubusercontent.com/mediar-ai/terminator/main/scripts/install.sh
mediar.ai/install.ps1 → 301 Redirect → https://raw.githubusercontent.com/mediar-ai/terminator/main/scripts/install.ps1
```

## Option 2: GitHub Pages (Free)
Create a GitHub Pages site at `mediar-ai.github.io/terminator`

### Setup:
1. Enable GitHub Pages in repo settings
2. Create `docs/index.html` with JavaScript redirect
3. Use custom domain if available

### Usage:
```bash
curl -fsSL https://mediar-ai.github.io/terminator/install.sh | bash
```

## Option 3: URL Shortener Service
Use services like bit.ly, short.io, or rebrand.ly

### Pros:
- Quick setup
- Analytics

### Cons:
- Third-party dependency
- Less professional appearance
- Potential trust issues

### Example:
```bash
curl -fsSL https://bit.ly/terminator-install | bash
```

## Option 4: Smart Installer Script
Create a single endpoint that detects the platform:

### Server Code (Node.js/Edge Function):
```javascript
export default function handler(req, res) {
  const userAgent = req.headers['user-agent'] || '';

  if (userAgent.includes('PowerShell')) {
    // Redirect to PS1 script
    res.redirect(301, 'https://raw.githubusercontent.com/.../install.ps1');
  } else {
    // Redirect to bash script
    res.redirect(301, 'https://raw.githubusercontent.com/.../install.sh');
  }
}
```

### Usage:
```bash
# Works on any platform
curl -fsSL https://get.mediar.ai | bash
irm https://get.mediar.ai | iex
```

## Recommendation

**Best approach: Option 1 with subdomain**

1. Set up `get.mediar.ai` subdomain
2. Configure redirects:
   - `get.mediar.ai` → Smart platform detection
   - `get.mediar.ai/sh` → Bash script
   - `get.mediar.ai/ps1` → PowerShell script

3. Final commands:
```bash
# Super short and memorable
curl -fsSL https://get.mediar.ai | bash  # macOS/Linux
irm https://get.mediar.ai | iex          # Windows
```

This gives you:
- ✅ Short, professional URLs
- ✅ Full control (no third-party dependency)
- ✅ Analytics if needed (via Cloudflare/server logs)
- ✅ Trust (your own domain)
- ✅ Easy to remember and share