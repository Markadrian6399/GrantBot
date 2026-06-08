# GrantBot 🤖💸

> AI-powered autonomous USDC payments for open-source contributors

## What it does

GrantBot monitors your GitHub repo for merged PRs and automatically pays
contributors in USDC — no human approval needed.

**Flow:**  
`GitHub Webhook → Rust/Axum Backend → Venice AI (reasoning) → 1Shot Relayer (execution) → Sepolia blockchain`

## Architecture

| Layer | Tech |
|-------|------|
| Backend | Rust + Axum + Tokio |
| Database | PostgreSQL + SQLx |
| AI Layer | Venice AI API (llama-3.3-70b) |
| Wallet | MetaMask Smart Accounts (ERC-7715) |
| Gas/Execution | 1Shot Permissionless Relayer (EIP-7710) |
| Chain | Ethereum Sepolia testnet |
| Frontend | React + Vite + TypeScript + Tailwind |

## Prize Tracks

- ✅ Best Agent
- ✅ Best x402 + ERC-7710
- ✅ Best use of Venice AI
- ✅ Best Use of 1Shot Permissionless Relayer
- ✅ Best A2A coordination

## Setup

### Prerequisites

- Rust 1.75+
- Node.js 18+
- PostgreSQL 14+
- ngrok (for webhook tunneling)

### 1. Clone & configure

```bash
git clone https://github.com/yourname/grantbot
cd grantbot
cp backend/.env.example backend/.env
# Fill in your API keys in backend/.env
```

### 2. Database setup

```bash
# Create DB + user
psql -U postgres -c "CREATE USER grantbot WITH PASSWORD 'grantbot123';"
psql -U postgres -c "CREATE DATABASE grantbot OWNER grantbot;"

# Run migrations
cd backend && cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```

### 3. Start the backend

```bash
cd backend
cargo run
# Backend starts on http://localhost:3001
```

### 4. Start the frontend

```bash
cd frontend
npm install
npm run dev
# Frontend starts on http://localhost:5173
```

### 5. Tunnel with ngrok

```bash
ngrok http 3001
# Use the ngrok URL as your GitHub webhook Payload URL
```

### 6. GitHub webhook setup

In your GitHub repo → Settings → Webhooks → Add webhook:
- **Payload URL:** `https://<ngrok-subdomain>.ngrok.io/webhook/github`
- **Content type:** `application/json`
- **Secret:** (generated and shown in the UI after registering a repo)
- **Events:** Pull requests only

## API Reference

| Method | Path | Description |
|--------|------|-------------|
| POST | `/repos` | Register a new repo |
| GET | `/repos/:id` | Get repo + stats |
| PUT | `/repos/:id/delegation` | Store ERC-7715 delegation |
| POST | `/repos/:id/contributors` | Add contributor |
| GET | `/repos/:id/contributors` | List contributors |
| GET | `/payments?repo_id=` | List payments |
| GET | `/payments/stats?repo_id=` | Get payment stats |
| POST | `/webhook/github` | GitHub webhook handler |
| POST | `/test/trigger-pr` | Simulate a merged PR |
| GET | `/test/pr-status/:id` | Poll payment status |

## Test the flow manually

```bash
# 1. Register a repo
curl -X POST http://localhost:3001/repos \
  -H "Content-Type: application/json" \
  -d '{"owner":"testuser","repo_name":"testrepo","payout_amount":5.0,"daily_cap":50.0,"owner_address":"0xYourAddress"}'

# 2. Add a contributor (use the repo id from step 1)
curl -X POST http://localhost:3001/repos/{id}/contributors \
  -H "Content-Type: application/json" \
  -d '{"github_username":"contributor1","wallet_address":"0xContributorAddress"}'

# 3. Simulate a merged PR
curl -X POST http://localhost:3001/test/trigger-pr \
  -H "Content-Type: application/json" \
  -d '{"repo_id":"{id}","pr_number":1,"pr_title":"Fix critical bug","contributor":"contributor1"}'

# 4. Poll status
curl http://localhost:3001/test/pr-status/{pr_event_id}
```

## Demo Video

[link here]
