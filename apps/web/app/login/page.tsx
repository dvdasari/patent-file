"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { api } from "@/lib/api-client";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setLoading(true);
    try {
      await api.login({ email, password });
      router.push("/projects");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen">
      {/* Left panel — branding */}
      <div className="hidden lg:flex lg:w-1/2 relative overflow-hidden" style={{ background: "linear-gradient(135deg, #0a0a0a 0%, #111113 50%, #18181b 100%)" }}>
        <div className="absolute inset-0" style={{ background: "radial-gradient(ellipse at 30% 50%, rgba(200, 169, 110, 0.06) 0%, transparent 70%)" }} />
        <div className="relative z-10 flex flex-col justify-between p-12 w-full">
          <div>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full" style={{ background: "var(--accent)" }} />
              <span className="text-xs font-mono uppercase tracking-[0.2em]" style={{ color: "var(--accent)" }}>
                Patent Draft Pro
              </span>
            </div>
          </div>

          <div className="space-y-6 max-w-md">
            <h1 className="text-4xl font-light leading-tight tracking-tight text-zinc-100">
              Draft patents with
              <br />
              <span className="font-medium" style={{ color: "var(--accent)" }}>precision AI</span>
            </h1>
            <p className="text-sm leading-relaxed text-zinc-500">
              Guided interview. AI-generated specifications.
              <br />
              IPO Form 2 compliant. Export-ready.
            </p>
            <div className="flex items-center gap-6 pt-4">
              {["Claims", "Specification", "Abstract", "Export"].map((item, i) => (
                <span key={item} className="text-xs text-zinc-600 font-mono animate-fade-in" style={{ animationDelay: `${i * 100}ms` }}>
                  {item}
                </span>
              ))}
            </div>
          </div>

          <div className="text-xs text-zinc-700 font-mono">
            Indian Patent Office &middot; Form 2 Format
          </div>
        </div>

        {/* Decorative lines */}
        <div className="absolute right-0 top-0 bottom-0 w-px" style={{ background: "var(--border)" }} />
        <div className="absolute bottom-24 right-12 w-48 h-px" style={{ background: "linear-gradient(90deg, transparent, var(--accent-dim), transparent)" }} />
      </div>

      {/* Right panel — form */}
      <div className="flex flex-1 items-center justify-center p-8" style={{ background: "var(--background)" }}>
        <div className="w-full max-w-sm animate-fade-in">
          {/* Mobile logo */}
          <div className="lg:hidden mb-12 text-center">
            <div className="flex items-center justify-center gap-2 mb-2">
              <div className="w-1.5 h-1.5 rounded-full" style={{ background: "var(--accent)" }} />
              <span className="text-xs font-mono uppercase tracking-[0.2em]" style={{ color: "var(--accent)" }}>
                Patent Draft Pro
              </span>
            </div>
          </div>

          <div className="space-y-1 mb-8">
            <h2 className="text-xl font-medium tracking-tight text-zinc-100">
              Sign in
            </h2>
            <p className="text-sm text-zinc-500">
              Enter your credentials to continue
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-5">
            {error && (
              <div className="rounded border px-3 py-2.5 text-sm animate-fade-in"
                style={{ borderColor: "rgba(239, 68, 68, 0.3)", background: "rgba(239, 68, 68, 0.05)", color: "#f87171" }}>
                {error}
              </div>
            )}

            <div className="space-y-1.5">
              <label htmlFor="email" className="text-xs font-medium uppercase tracking-wider text-zinc-400">
                Email
              </label>
              <input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                autoComplete="email"
                autoFocus
                className="w-full rounded border bg-transparent px-3 py-2.5 text-sm text-zinc-100 transition-colors placeholder:text-zinc-600 focus:outline-none"
                style={{ borderColor: "var(--border)" }}
                onFocus={(e) => e.target.style.borderColor = "var(--accent-dim)"}
                onBlur={(e) => e.target.style.borderColor = "var(--border)"}
                placeholder="you@example.com"
              />
            </div>

            <div className="space-y-1.5">
              <label htmlFor="password" className="text-xs font-medium uppercase tracking-wider text-zinc-400">
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                autoComplete="current-password"
                className="w-full rounded border bg-transparent px-3 py-2.5 text-sm text-zinc-100 transition-colors placeholder:text-zinc-600 focus:outline-none"
                style={{ borderColor: "var(--border)" }}
                onFocus={(e) => e.target.style.borderColor = "var(--accent-dim)"}
                onBlur={(e) => e.target.style.borderColor = "var(--border)"}
                placeholder="&bull;&bull;&bull;&bull;&bull;&bull;&bull;&bull;"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full rounded px-4 py-2.5 text-sm font-medium transition-all duration-200 disabled:opacity-40"
              style={{
                background: loading ? "var(--surface-raised)" : "var(--accent)",
                color: loading ? "var(--muted)" : "var(--background)",
              }}
              onMouseEnter={(e) => { if (!loading) e.currentTarget.style.opacity = "0.9"; }}
              onMouseLeave={(e) => { e.currentTarget.style.opacity = "1"; }}
            >
              {loading ? (
                <span className="flex items-center justify-center gap-2">
                  <span className="inline-block w-3 h-3 border border-zinc-500 border-t-transparent rounded-full animate-spin" />
                  Signing in...
                </span>
              ) : "Sign in"}
            </button>
          </form>

          <p className="mt-8 text-center text-xs text-zinc-700">
            Access is by invitation only
          </p>
        </div>
      </div>
    </div>
  );
}
