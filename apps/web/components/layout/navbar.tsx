"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useAuth } from "@/hooks/use-auth";
import { useRouter } from "next/navigation";

export function Navbar() {
  const { user, logout } = useAuth();
  const router = useRouter();
  const pathname = usePathname();

  async function handleLogout() {
    await logout();
    router.push("/login");
  }

  const isActive = (path: string) => pathname.startsWith(path);

  return (
    <nav className="border-b" style={{ borderColor: "var(--border)", background: "var(--surface)" }}>
      <div className="mx-auto flex h-12 max-w-5xl items-center justify-between px-6">
        <div className="flex items-center gap-8">
          <Link href="/projects" className="flex items-center gap-2 group">
            <div className="w-1.5 h-1.5 rounded-full transition-colors group-hover:scale-110" style={{ background: "var(--accent)" }} />
            <span className="text-xs font-mono uppercase tracking-[0.15em] transition-colors" style={{ color: "var(--accent)" }}>
              Patent Draft Pro
            </span>
          </Link>
          <div className="flex items-center gap-1">
            {[
              { href: "/projects", label: "Projects" },
              { href: "/search", label: "Prior Art" },
              { href: "/account", label: "Account" },
            ].map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="px-3 py-1.5 rounded text-xs font-medium transition-colors"
                style={{
                  color: isActive(link.href) ? "var(--foreground)" : "var(--muted)",
                  background: isActive(link.href) ? "var(--surface-raised)" : "transparent",
                }}
              >
                {link.label}
              </Link>
            ))}
          </div>
        </div>

        {user && (
          <div className="flex items-center gap-4">
            <span className="text-xs text-zinc-500 font-mono">{user.email}</span>
            <button
              onClick={handleLogout}
              className="text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
            >
              Sign out
            </button>
          </div>
        )}
      </div>
    </nav>
  );
}
