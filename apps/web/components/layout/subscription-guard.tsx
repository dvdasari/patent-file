"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/use-auth";

export function SubscriptionGuard({ children }: { children: React.ReactNode }) {
  const { hasSubscription, loading, isAuthenticated } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!loading && isAuthenticated && !hasSubscription) {
      router.replace("/subscribe");
    }
  }, [loading, isAuthenticated, hasSubscription, router]);

  if (loading || !hasSubscription) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="text-sm text-zinc-400">Loading...</div>
      </div>
    );
  }

  return <>{children}</>;
}
