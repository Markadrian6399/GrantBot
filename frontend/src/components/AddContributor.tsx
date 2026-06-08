import { useState, useEffect } from 'react'
import { api, type Contributor } from '../hooks/useGrantBot'

interface Props {
  repoId: string
}

export function AddContributor({ repoId }: Props) {
  const [form, setForm] = useState({ github_username: '', wallet_address: '' })
  const [contributors, setContributors] = useState<Contributor[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [refresh, setRefresh] = useState(0)

  useEffect(() => {
    let cancelled = false
    api.listContributors(repoId).then((data) => {
      if (!cancelled) setContributors(data)
    }).catch(() => {/* ignore */})
    return () => { cancelled = true }
  }, [repoId, refresh])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      await api.addContributor(repoId, form)
      setForm({ github_username: '', wallet_address: '' })
      setRefresh((n) => n + 1)
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to add contributor')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="bg-slate-800/60 border border-slate-700 rounded-xl p-5">
      <h2 className="text-white font-semibold text-lg mb-4">Add Contributor</h2>
      <form onSubmit={handleSubmit} className="space-y-3 mb-4">
        <input
          className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-purple-500"
          placeholder="GitHub Username"
          value={form.github_username}
          onChange={(e) => setForm((f) => ({ ...f, github_username: e.target.value }))}
          required
        />
        <input
          className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-2 text-sm text-white placeholder-slate-500 focus:outline-none focus:border-purple-500 font-mono"
          placeholder="Ethereum Wallet Address (0x...)"
          value={form.wallet_address}
          onChange={(e) => setForm((f) => ({ ...f, wallet_address: e.target.value }))}
          required
        />
        {error && <p className="text-red-400 text-xs">{error}</p>}
        <button
          type="submit"
          disabled={loading}
          className="w-full bg-purple-600 hover:bg-purple-700 disabled:opacity-50 text-white font-semibold py-2 rounded-lg transition-colors text-sm"
        >
          {loading ? 'Adding...' : 'Add Contributor'}
        </button>
      </form>

      {contributors.length > 0 && (
        <div className="space-y-2">
          <p className="text-slate-400 text-xs uppercase tracking-wider font-semibold mb-2">
            Registered Contributors
          </p>
          {contributors.map((c) => (
            <div
              key={c.id}
              className="flex items-center justify-between bg-slate-900/80 rounded-lg px-3 py-2"
            >
              <span className="text-purple-400 text-sm">@{c.github_username}</span>
              <span className="text-slate-500 font-mono text-xs">
                {c.wallet_address.slice(0, 8)}...{c.wallet_address.slice(-6)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
