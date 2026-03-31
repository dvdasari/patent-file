"use client";

import { useState, useRef } from "react";
import { api } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";

type PatentResult = {
  patent_number: string;
  title: string;
  abstract_text: string | null;
  applicants: string[];
  inventors: string[];
  filing_date: string | null;
  publication_date: string | null;
  jurisdiction: string;
  url: string | null;
};

type SearchResponse = {
  results: PatentResult[];
  total: number;
  page: number;
  per_page: number;
  query: string;
};

function formatDate(d: string | null) {
  if (!d) return null;
  try {
    return new Date(d).toLocaleDateString("en-IN", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return d;
  }
}

function PatentCard({ result }: { result: PatentResult }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      className="rounded border p-5 transition-colors"
      style={{ borderColor: "var(--border)", background: "var(--surface)" }}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1.5 flex-wrap">
            <span
              className="text-xs font-mono px-1.5 py-0.5 rounded"
              style={{
                background: "rgba(200,169,110,0.1)",
                color: "var(--accent)",
                border: "1px solid rgba(200,169,110,0.2)",
              }}
            >
              {result.patent_number}
            </span>
            <span
              className="text-xs px-1.5 py-0.5 rounded"
              style={{ background: "var(--surface-raised)", color: "var(--muted)" }}
            >
              {result.jurisdiction}
            </span>
          </div>

          <h3 className="text-sm font-medium text-zinc-100 leading-snug mb-2">
            {result.title}
          </h3>

          <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-zinc-500 mb-3">
            {result.applicants.length > 0 && (
              <span>
                <span className="text-zinc-600">Applicant: </span>
                {result.applicants.join("; ")}
              </span>
            )}
            {result.inventors.length > 0 && (
              <span>
                <span className="text-zinc-600">Inventors: </span>
                {result.inventors.join(", ")}
              </span>
            )}
          </div>

          <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-zinc-600">
            {result.filing_date && (
              <span>Filed: {formatDate(result.filing_date)}</span>
            )}
            {result.publication_date && (
              <span>Published: {formatDate(result.publication_date)}</span>
            )}
          </div>

          {result.abstract_text && (
            <div className="mt-3">
              <p
                className="text-xs leading-relaxed"
                style={{ color: "var(--muted)" }}
              >
                {expanded
                  ? result.abstract_text
                  : result.abstract_text.slice(0, 180) +
                    (result.abstract_text.length > 180 ? "…" : "")}
              </p>
              {result.abstract_text.length > 180 && (
                <button
                  onClick={() => setExpanded((v) => !v)}
                  className="text-xs mt-1 transition-colors"
                  style={{ color: "var(--accent)" }}
                >
                  {expanded ? "Show less" : "Show more"}
                </button>
              )}
            </div>
          )}
        </div>

        {result.url && (
          <a
            href={result.url}
            target="_blank"
            rel="noopener noreferrer"
            className="shrink-0 text-xs px-3 py-1.5 rounded border transition-colors"
            style={{
              borderColor: "var(--border)",
              color: "var(--muted)",
            }}
          >
            View ↗
          </a>
        )}
      </div>
    </div>
  );
}

function SearchContent() {
  const [query, setQuery] = useState("");
  const [response, setResponse] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const inputRef = useRef<HTMLInputElement>(null);

  async function runSearch(q: string, p: number) {
    if (!q.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.searchPatents({ q, page: p, per_page: 10 });
      setResponse(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Search failed");
    } finally {
      setLoading(false);
    }
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setPage(1);
    runSearch(query, 1);
  }

  function handlePageChange(newPage: number) {
    setPage(newPage);
    runSearch(response?.query ?? query, newPage);
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  const totalPages = response
    ? Math.ceil(response.total / response.per_page)
    : 0;

  return (
    <div className="mx-auto max-w-3xl px-6 py-10 animate-fade-in">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-lg font-medium tracking-tight text-zinc-100">
          Prior Art Search
        </h1>
        <p className="text-xs text-zinc-500 mt-1">
          Search Indian patents — powered by the IPO database
        </p>
      </div>

      {/* Search form */}
      <form onSubmit={handleSubmit} className="mb-8">
        <div className="flex gap-2">
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="e.g. solar panel efficiency, natural language processing, drug delivery…"
            className="flex-1 rounded border px-4 py-2.5 text-sm outline-none transition-colors"
            style={{
              background: "var(--surface)",
              borderColor: "var(--border)",
              color: "var(--foreground)",
            }}
            onFocus={(e) =>
              (e.currentTarget.style.borderColor = "var(--accent-dim)")
            }
            onBlur={(e) =>
              (e.currentTarget.style.borderColor = "var(--border)")
            }
            autoFocus
          />
          <button
            type="submit"
            disabled={loading || !query.trim()}
            className="rounded px-5 py-2.5 text-xs font-medium transition-all disabled:opacity-40"
            style={{
              background: "var(--accent)",
              color: "var(--background)",
            }}
          >
            {loading ? "Searching…" : "Search"}
          </button>
        </div>
      </form>

      {/* Error */}
      {error && (
        <div
          className="rounded border px-4 py-3 text-sm mb-6"
          style={{
            borderColor: "rgba(239,68,68,0.3)",
            background: "rgba(239,68,68,0.05)",
            color: "#f87171",
          }}
        >
          {error}
        </div>
      )}

      {/* Loading skeleton */}
      {loading && (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div
              key={i}
              className="rounded border p-5 animate-pulse"
              style={{ borderColor: "var(--border)", background: "var(--surface)" }}
            >
              <div
                className="h-3 w-24 rounded mb-3"
                style={{ background: "var(--border)" }}
              />
              <div
                className="h-4 w-3/4 rounded mb-2"
                style={{ background: "var(--surface-raised)" }}
              />
              <div
                className="h-3 w-1/2 rounded"
                style={{ background: "var(--border)" }}
              />
            </div>
          ))}
        </div>
      )}

      {/* Results */}
      {!loading && response && (
        <>
          <div className="flex items-center justify-between mb-4">
            <p className="text-xs text-zinc-500">
              {response.total === 0
                ? "No results found"
                : `${response.total} result${response.total !== 1 ? "s" : ""} for "${response.query}"`}
            </p>
            {totalPages > 1 && (
              <p className="text-xs text-zinc-600">
                Page {page} of {totalPages}
              </p>
            )}
          </div>

          {response.results.length === 0 ? (
            <div
              className="rounded border border-dashed p-12 text-center"
              style={{ borderColor: "var(--border)" }}
            >
              <p className="text-sm text-zinc-400 mb-1">No patents found</p>
              <p className="text-xs text-zinc-600">
                Try broader keywords or different terminology
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {response.results.map((r, i) => (
                <div
                  key={r.patent_number + i}
                  className="animate-fade-in"
                  style={{ animationDelay: `${i * 40}ms` }}
                >
                  <PatentCard result={r} />
                </div>
              ))}
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2 mt-8">
              <button
                onClick={() => handlePageChange(page - 1)}
                disabled={page <= 1}
                className="px-3 py-1.5 rounded border text-xs transition-colors disabled:opacity-30"
                style={{ borderColor: "var(--border)", color: "var(--muted)" }}
              >
                ← Prev
              </button>
              {Array.from({ length: Math.min(totalPages, 7) }, (_, i) => {
                const p = i + 1;
                return (
                  <button
                    key={p}
                    onClick={() => handlePageChange(p)}
                    className="w-8 h-8 rounded text-xs font-medium transition-colors"
                    style={{
                      background:
                        p === page ? "var(--accent)" : "transparent",
                      color:
                        p === page ? "var(--background)" : "var(--muted)",
                      border:
                        p === page
                          ? "none"
                          : "1px solid var(--border)",
                    }}
                  >
                    {p}
                  </button>
                );
              })}
              <button
                onClick={() => handlePageChange(page + 1)}
                disabled={page >= totalPages}
                className="px-3 py-1.5 rounded border text-xs transition-colors disabled:opacity-30"
                style={{ borderColor: "var(--border)", color: "var(--muted)" }}
              >
                Next →
              </button>
            </div>
          )}
        </>
      )}

      {/* Empty state (initial) */}
      {!loading && !response && !error && (
        <div
          className="rounded border border-dashed p-16 text-center"
          style={{ borderColor: "var(--border)" }}
        >
          <div
            className="mx-auto w-12 h-12 rounded-full flex items-center justify-center mb-4"
            style={{ background: "rgba(200, 169, 110, 0.08)" }}
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              style={{ color: "var(--accent)" }}
            >
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.35-4.35" />
            </svg>
          </div>
          <h2 className="text-sm font-medium text-zinc-300 mb-1">
            Search Indian patents
          </h2>
          <p className="text-xs text-zinc-600">
            Enter keywords to find prior art from the Indian Patent Office database
          </p>
        </div>
      )}
    </div>
  );
}

export default function PatentSearchPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <SearchContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
