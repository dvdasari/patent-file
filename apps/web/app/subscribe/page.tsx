"use client";

import { AuthGuard } from "@/components/layout/auth-guard";
import { Navbar } from "@/components/layout/navbar";

function SubscribeContent() {
  return (
    <div className="mx-auto max-w-sm px-6 py-20 animate-fade-in">
      <div className="text-center mb-10">
        <div className="flex items-center justify-center gap-2 mb-4">
          <div className="w-1.5 h-1.5 rounded-full" style={{ background: "var(--accent)" }} />
          <span className="text-xs font-mono uppercase tracking-[0.15em]" style={{ color: "var(--accent)" }}>
            Patent Draft Pro
          </span>
        </div>
        <h1 className="text-xl font-medium tracking-tight text-zinc-100 mb-2">
          Subscribe to continue
        </h1>
        <p className="text-sm text-zinc-500">
          Full access to AI-powered patent drafting
        </p>
      </div>

      <div className="rounded border p-6 mb-6" style={{ borderColor: "var(--accent-dim)", background: "var(--surface)" }}>
        <div className="text-center mb-6">
          <span className="text-3xl font-light text-zinc-100">&#8377; TBD</span>
          <span className="text-sm text-zinc-500">/month</span>
        </div>

        <div className="space-y-3 mb-6">
          {[
            "Unlimited patent drafts",
            "AI generation (Claude)",
            "PDF + DOCX export",
            "IPO Form 2 compliant",
            "Section version history",
          ].map((feature) => (
            <div key={feature} className="flex items-center gap-2.5">
              <span style={{ color: "var(--accent)" }}>✓</span>
              <span className="text-sm text-zinc-300">{feature}</span>
            </div>
          ))}
        </div>

        <button
          className="w-full rounded py-2.5 text-sm font-medium transition-all"
          style={{ background: "var(--accent)", color: "var(--background)" }}
        >
          Subscribe with Razorpay
        </button>
        <p className="mt-3 text-center text-xs text-zinc-600">
          UPI &middot; Cards &middot; Netbanking
        </p>
      </div>
    </div>
  );
}

export default function SubscribePage() {
  return (
    <AuthGuard>
      <Navbar />
      <SubscribeContent />
    </AuthGuard>
  );
}
