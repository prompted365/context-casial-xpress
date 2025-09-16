# Railway Port Configuration for MOP

## Current Setup

### For Railway Deployment:
- **Internal Port**: Your app listens on PORT environment variable (defaults to 8081)
- **Public Domain Port**: You can keep this at 8080 (Railway handles the mapping)

## How It Works

1. **Your App**: Listens on `PORT` environment variable (8081 internally)
2. **Railway**: Maps public port 8080 → internal port 8081
3. **Users**: Access via `https://context-casial-xpress-production.up.railway.app` (port 443 HTTPS)

## Configuration Steps

### Option 1: Keep Public Port 8080 (Recommended)
- **Railway Domain Settings**: Keep target port as 8080
- **Environment Variables**: Add `PORT=8081` in Railway dashboard
- **Result**: Railway maps public:8080 → internal:8081

### Option 2: Change Everything to 8081
- **Railway Domain Settings**: Change target port to 8081
- **Environment Variables**: PORT will auto-set to 8081
- **Result**: Consistent port everywhere

## For Smithery Integration
Smithery will set its own PORT (usually 8081), so our startup script respects the PORT env var:
```bash
SERVER_PORT=${PORT:-8081}  # Uses PORT if set, otherwise 8081
```

## Quick Answer
**No**, you don't need to change the Railway public domain port to 8081. Keep it at 8080 and just ensure your app listens on the PORT environment variable internally.