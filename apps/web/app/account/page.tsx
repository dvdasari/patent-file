"use client";

import { useAuth } from "@/hooks/use-auth";
import { AuthGuard } from "@/components/layout/auth-guard";
import { Navbar } from "@/components/layout/navbar";

function AccountContent() {
  const { user } = useAuth();
  if (!user) return null;

  return (
    <div className="mx-auto max-w-md px-6 py-10 animate-fade-in">
      <h1 className="text-lg font-medium tracking-tight text-zinc-100 mb-8">Account</h1>

      <div className="space-y-4">
        <div className="rounded border p-5" style={{ borderColor: "var(--border)", background: "var(--surface)" }}>
          <h2 className="text-xs font-medium uppercase tracking-wider mb-4" style={{ color: "var(--muted)" }}>
            Profile
          </h2>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs text-zinc-500">Name</span>
              <span className="text-sm text-zinc-200">{user.full_name}</span>
            </div>
            <div className="h-px" style={{ background: "var(--border-subtle)" }} />
            <div className="flex items-center justify-between">
              <span className="text-xs text-zinc-500">Email</span>
              <span className="text-sm text-zinc-200 font-mono">{user.email}</span>
            </div>
          </div>
        </div>

        <div className="rounded border p-5" style={{ borderColor: "var(--border)", background: "var(--surface)" }}>
          <h2 className="text-xs font-medium uppercase tracking-wider mb-4" style={{ color: "var(--muted)" }}>
            Subscription
          </h2>
          <div className="flex items-center justify-between">
            <span className="text-xs text-zinc-500">Status</span>
            <span
              className="inline-flex items-center gap-1.5 rounded px-2 py-0.5 text-xs font-medium"
              style={{
                color: user.has_active_subscription ? "#4ade80" : "#f87171",
                background: user.has_active_subscription ? "rgba(74, 222, 128, 0.1)" : "rgba(248, 113, 113, 0.1)",
              }}
            >
              <div
                className="w-1.5 h-1.5 rounded-full"
                style={{ background: user.has_active_subscription ? "#4ade80" : "#f87171" }}
              />
              {user.has_active_subscription ? "Active" : "Inactive"}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function AccountPage() {
  return (
    <AuthGuard>
      <Navbar />
      <AccountContent />
    </AuthGuard>
  );
}
