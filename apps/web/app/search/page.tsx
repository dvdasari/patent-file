"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { api, PriorArtSearch } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { SearchForm } from "@/components/search/search-form";

function SearchContent() {
  const router = useRouter();
  const [searches, setSearches] = useState<PriorArtSearch[]>([]);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    api
      .listSearches()
      .then((data) => {
        setSearches(data || []);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  async function handleSearch(params: {
    query: string;
    ipc_classification?: string;
    applicant?: string;
    date_from?: string;
    date_to?: string;
    include_npl?: boolean;
  }) {
    setSubmitting(true);
    try {
      const res = await api.createSearch(params);
      router.push(`/search/${res.search.id}`);
    } catch (err) {
      console.error("Search failed:", err);
      setSubmitting(false);
    }
  }

  const statusColor = (status: string) => {
    switch (status) {
      case "complete":
        return "var(--accent)";
      case "searching":
      case "analyzing":
        return "#f59e0b";
      case "failed":
        return "#ef4444";
      default:
        return "var(--muted)";
    }
  };

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <div className="mb-8">
        <h1
          className="text-lg font-medium mb-1"
          style={{ color: "var(--foreground)" }}
        >
          Prior Art Search
        </h1>
        <p className="text-xs" style={{ color: "var(--muted)" }}>
          Search Indian patent databases and global sources with AI-powered
          relevance ranking
        </p>
      </div>

      <SearchForm onSubmit={handleSearch} submitting={submitting} />

      <div className="mt-10">
        <h2
          className="text-sm font-medium mb-4"
          style={{ color: "var(--foreground)" }}
        >
          Recent Searches
        </h2>

        {loading ? (
          <div className="flex items-center gap-3 py-8">
            <div className="w-3 h-3 border border-zinc-600 border-t-transparent rounded-full animate-spin" />
            <span className="text-xs" style={{ color: "var(--muted)" }}>
              Loading searches...
            </span>
          </div>
        ) : searches.length === 0 ? (
          <p className="text-xs py-8" style={{ color: "var(--muted)" }}>
            No searches yet. Start your first prior art search above.
          </p>
        ) : (
          <div className="space-y-2">
            {searches.map((s) => (
              <button
                key={s.id}
                onClick={() => router.push(`/search/${s.id}`)}
                className="w-full text-left p-4 rounded border transition-colors hover:border-zinc-600"
                style={{
                  borderColor: "var(--border)",
                  background: "var(--surface)",
                }}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <p
                      className="text-sm font-medium truncate"
                      style={{ color: "var(--foreground)" }}
                    >
                      {s.query_text}
                    </p>
                    <div className="flex items-center gap-3 mt-1">
                      <span className="text-xs" style={{ color: "var(--muted)" }}>
                        {new Date(s.created_at).toLocaleDateString()}
                      </span>
                      {s.ipc_classification && (
                        <span className="text-xs" style={{ color: "var(--muted)" }}>
                          IPC: {s.ipc_classification}
                        </span>
                      )}
                      {s.include_npl && (
                        <span className="text-xs" style={{ color: "var(--muted)" }}>
                          +NPL
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs" style={{ color: "var(--muted)" }}>
                      {s.result_count} results
                    </span>
                    <span
                      className="text-xs font-medium px-2 py-0.5 rounded"
                      style={{
                        color: statusColor(s.status),
                        background: `${statusColor(s.status)}15`,
                      }}
                    >
                      {s.status}
                    </span>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default function SearchPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <SearchContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
