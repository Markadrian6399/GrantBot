import axios from 'axios'

const API = axios.create({
  baseURL: import.meta.env.VITE_BACKEND_URL || 'http://localhost:3001',
})

export interface Repo {
  id: string
  owner: string
  repo_name: string
  payout_amount: number
  daily_cap: number
  owner_address: string
  webhook_secret: string
  delegation_hex: string | null
}

export interface Contributor {
  id: string
  github_username: string
  wallet_address: string
  repo_id: string
}

export interface Payment {
  id: string
  pr_event_id: string
  amount: number
  tx_hash: string | null
  venice_reason: string
  created_at: string
  pr_number: number
  pr_title: string
  contributor: string
  status: string
  merged_at: string
  repo_id: string
}

export interface Stats {
  total_paid: number
  pr_count: number
  today_spend: number
  contributor_count: number
}

export interface PrStatus {
  status: string
  payment?: {
    amount: number
    tx_hash: string | null
    venice_reason: string
  }
}

// All API functions are module-level so they're stable references across renders.
export const api = {
  registerRepo: async (data: {
    owner: string
    repo_name: string
    payout_amount: number
    daily_cap: number
    owner_address: string
  }) => {
    const res = await API.post('/repos', data)
    return res.data as { repo: Repo; webhook_url: string; webhook_secret: string }
  },

  addContributor: async (
    repoId: string,
    data: { github_username: string; wallet_address: string }
  ) => {
    const res = await API.post(`/repos/${repoId}/contributors`, data)
    return res.data as Contributor
  },

  listContributors: async (repoId: string) => {
    const res = await API.get(`/repos/${repoId}/contributors`)
    return res.data as Contributor[]
  },

  storeDelegation: async (repoId: string, delegationHex: string) => {
    await API.put(`/repos/${repoId}/delegation`, { delegation_hex: delegationHex })
  },

  getPayments: async (repoId: string) => {
    const res = await API.get(`/payments?repo_id=${repoId}`)
    return res.data as Payment[]
  },

  getStats: async (repoId: string) => {
    const res = await API.get(`/payments/stats?repo_id=${repoId}`)
    return res.data as Stats
  },

  triggerTestPR: async (data: {
    repo_id: string
    pr_number: number
    pr_title: string
    contributor: string
  }) => {
    const res = await API.post('/test/trigger-pr', data)
    return res.data as { pr_event_id: string }
  },

  pollPRStatus: async (prEventId: string) => {
    const res = await API.get(`/test/pr-status/${prEventId}`)
    return res.data as PrStatus
  },
}

// Hook kept for backward compat — just returns the stable api object.
export function useGrantBot() {
  return api
}
