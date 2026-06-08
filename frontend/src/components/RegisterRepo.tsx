import { useState } from 'react'
import { api, type Repo } from '../hooks/useGrantBot'

interface Props {
  onRegistered: (repo: Repo) => void
  ownerAddress: string
}

export function RegisterRepo({ onRegistered, ownerAddress }: Props) {
  const { registerRepo } = api
  const [form, setForm] = useState({
    owner: '',
    repo_name: '',
    payout_amount: 5,
    daily_cap: 50,
  })
  const [result, setResult] = useState<{ webhook_secret: string; webhook_url: string } | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      const res = await registerRepo({ ...form, owner_address: ownerAddress })
      setResult({ webhook_secret: res.webhook_secret, webhook_url: res.webhook_url })
      onRegistered(res.repo)
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Registration failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-5">
      <h2 className="text-white font-semibold text-lg mb-4">Register Repository</h2>
      <form onSubmit={handleSubmit} className="space-y-3">
        <div className="grid grid-cols-2 gap-3">
          <input
            className="bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-purple-500"
            placeholder="GitHub Owner"
            value={form.owner}
            onChange={(e) => setForm((f) => ({ ...f, owner: e.target.value }))}
            required
          />
          <input
            className="bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-purple-500"
            placeholder="Repo Name"
            value={form.repo_name}
            onChange={(e) => setForm((f) => ({ ...f, repo_name: e.target.value }))}
            required
          />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-slate-400 block mb-1">USDC per PR</label>
            <input
              type="number"
              className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-purple-500"
              value={form.payout_amount}
              onChange={(e) => setForm((f) => ({ ...f, payout_amount: Number(e.target.value) }))}
              min={0.01}
              step={0.01}
              required
            />
          </div>
          <div>
            <label className="text-xs text-slate-400 block mb-1">Daily Cap (USDC)</label>
            <input
              type="number"
              className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-purple-500"
              value={form.daily_cap}
              onChange={(e) => setForm((f) => ({ ...f, daily_cap: Number(e.target.value) }))}
              min={0.01}
              step={0.01}
              required
            />
          </div>
        </div>
        {error && <p className="text-red-400 text-xs">{error}</p>}
        <button
          type="submit"
          disabled={loading}
          className="w-full bg-purple-600 hover:bg-purple-700 disabled:opacity-50 text-white font-semibold py-2 rounded-lg transition-colors text-sm"
        >
          {loading ? 'Registering...' : 'Register Repo'}
        </button>
      </form>

      {result && (
        <div className="mt-4 bg-green-900/30 border border-green-700/50 rounded-lg p-4">
          <p className="text-green-400 font-semibold text-sm mb-3">Repo registered!</p>
          <p className="text-xs text-slate-400 mb-2 font-semibold uppercase tracking-wider">
            GitHub Webhook Setup
          </p>
          <div className="space-y-2 text-xs font-mono">
            <div className="bg-slate-900 rounded p-2">
              <span className="text-slate-500">Payload URL: </span>
              <span className="text-green-400">{'<ngrok-url>/webhook/github'}</span>
            </div>
            <div className="bg-slate-900 rounded p-2">
              <span className="text-slate-500">Content type: </span>
              <span className="text-white">application/json</span>
            </div>
            <div className="bg-slate-900 rounded p-2">
              <span className="text-slate-500">Secret: </span>
              <span className="text-yellow-400 break-all">{result.webhook_secret}</span>
            </div>
            <div className="bg-slate-900 rounded p-2">
              <span className="text-slate-500">Events: </span>
              <span className="text-white">Pull requests only</span>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
