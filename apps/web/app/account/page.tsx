"use client";

import { useAuth } from "@/hooks/use-auth";
import { AuthGuard } from "@/components/layout/auth-guard";
import { Navbar } from "@/components/layout/navbar";

function AccountContent() {
  const { user } = useAuth();

  if (!user) return null;

  return (
    <div className="mx-auto max-w-md px-4 py-8">
      <h1 className="text-lg font-semibold text-zinc-100 mb-6">Account</h1>

      <div className="space-y-4">
        <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-4">
          <h2 className="text-sm font-medium text-zinc-300 mb-3">Profile</h2>
          <div className="space-y-2 text-sm">
            <div>
              <span className="text-zinc-500">Name: </span>
              <span className="text-zinc-200">{user.full_name}</span>
            </div>
            <div>
              <span className="text-zinc-500">Email: </span>
              <span className="text-zinc-200">{user.email}</span>
            </div>
          </div>
        </div>

        <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-4">
          <h2 className="text-sm font-medium text-zinc-300 mb-3">Subscription</h2>
          <div className="text-sm">
            <span
              className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${
                user.has_active_subscription
                  ? "bg-green-900/50 text-green-400"
                  : "bg-red-900/50 text-red-400"
              }`}
            >
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
