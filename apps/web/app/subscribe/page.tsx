"use client";

import { useState } from "react";
import { AuthGuard } from "@/components/layout/auth-guard";
import { Navbar } from "@/components/layout/navbar";

function SubscribeContent() {
  const [loading] = useState(false);

  return (
    <div className="mx-auto max-w-md px-4 py-16">
      <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-8 text-center">
        <h1 className="text-xl font-semibold text-zinc-100 mb-2">
          Patent Draft Pro
        </h1>
        <p className="text-sm text-zinc-400 mb-6">
          AI-powered patent drafting for Indian Patent Office
        </p>

        <div className="rounded-lg bg-zinc-800 p-6 mb-6">
          <p className="text-3xl font-bold text-zinc-100">
            &#8377; TBD<span className="text-sm font-normal text-zinc-400">/month</span>
          </p>
          <ul className="mt-4 space-y-2 text-sm text-zinc-300 text-left">
            <li>Unlimited patent drafts</li>
            <li>AI-powered generation (Claude)</li>
            <li>PDF + DOCX export (IPO Form 2)</li>
            <li>Section editor with version history</li>
          </ul>
        </div>

        <button
          disabled={loading}
          className="w-full rounded-md bg-zinc-100 px-4 py-3 text-sm font-semibold text-zinc-900 hover:bg-zinc-200 disabled:opacity-50"
        >
          {loading ? "Setting up..." : "Subscribe with Razorpay"}
        </button>
        <p className="mt-3 text-xs text-zinc-500">
          UPI, Cards, Netbanking accepted
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
