"use client";

import Link from "next/link";
import { useAuth } from "@/hooks/use-auth";
import { useRouter } from "next/navigation";

export function Navbar() {
  const { user, logout } = useAuth();
  const router = useRouter();

  async function handleLogout() {
    await logout();
    router.push("/login");
  }

  return (
    <nav className="border-b border-zinc-800 bg-zinc-950">
      <div className="mx-auto flex h-14 max-w-5xl items-center justify-between px-4">
        <div className="flex items-center gap-6">
          <Link
            href="/projects"
            className="text-sm font-semibold text-zinc-100"
          >
            Patent Draft Pro
          </Link>
          <Link
            href="/projects"
            className="text-sm text-zinc-400 hover:text-zinc-100"
          >
            Projects
          </Link>
          <Link
            href="/account"
            className="text-sm text-zinc-400 hover:text-zinc-100"
          >
            Account
          </Link>
        </div>

        {user && (
          <div className="flex items-center gap-4">
            <span className="text-sm text-zinc-400">{user.email}</span>
            <button
              onClick={handleLogout}
              className="text-sm text-zinc-400 hover:text-zinc-100"
            >
              Sign out
            </button>
          </div>
        )}
      </div>
    </nav>
  );
}
