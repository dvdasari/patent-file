"use client";

import { useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { api } from "@/lib/api-client";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:5012";

export default function LoginPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState(searchParams.get("error") || "");
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

  function handleOAuth(provider: string) {
    window.location.href = `${API_URL}/api/auth/oauth/${provider}`;
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

          <div className="relative my-6">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t" style={{ borderColor: "var(--border)" }} />
            </div>
            <div className="relative flex justify-center text-xs">
              <span className="px-2 text-zinc-600" style={{ background: "var(--background)" }}>
                or continue with
              </span>
            </div>
          </div>

          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => handleOAuth("google")}
              className="flex-1 flex items-center justify-center gap-2 rounded border px-4 py-2.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-800/50"
              style={{ borderColor: "var(--border)" }}
            >
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
              </svg>
              Google
            </button>
            <button
              type="button"
              onClick={() => handleOAuth("linkedin")}
              className="flex-1 flex items-center justify-center gap-2 rounded border px-4 py-2.5 text-sm text-zinc-300 transition-colors hover:bg-zinc-800/50"
              style={{ borderColor: "var(--border)" }}
            >
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="#0A66C2">
                <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433a2.062 2.062 0 0 1-2.063-2.065 2.064 2.064 0 1 1 2.063 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
              </svg>
              LinkedIn
            </button>
          </div>

          <p className="mt-8 text-center text-xs text-zinc-700">
            Access is by invitation only
          </p>
        </div>
      </div>
    </div>
  );
}
