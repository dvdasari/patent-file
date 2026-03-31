"use client";

import { useEffect, useState, useCallback } from "react";
import { useParams } from "next/navigation";
import { api, SearchResponse, PriorArtResult } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { ResultCard } from "@/components/search/result-card";
import Link from "next/link";

function SearchResultsContent() {
  const params = useParams();
  const searchId = params.id as string;
  const [data, setData] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [generatingReport, setGeneratingReport] = useState(false);
  const [reportUrl, setReportUrl] = useState<string | null>(null);

  const fetchSearch = useCallback(async () => {
    try {
      const res = await api.getSearch(searchId);
      setData(res);
      setLoading(false);
    } catch {
      setLoading(false);
    }
  }, [searchId]);

  useEffect(() => {
    fetchSearch();
  }, [fetchSearch]);

  // Poll while search is in progress
  useEffect(() => {
    if (
      !data ||
      data.search.status === "complete" ||
      data.search.status === "failed"
    )
      return;

    const interval = setInterval(fetchSearch, 3000);
    return () => clearInterval(interval);
  }, [data, fetchSearch]);

  async function handleGenerateReport() {
    if (!data) return;
    setGeneratingReport(true);
    try {
      const report = await api.generateSearchReport(data.search.id);
      const download = await api.downloadSearchReport(report.id);
      setReportUrl(download.url);
    } catch (err) {
      console.error("Report generation failed:", err);
    } finally {
      setGeneratingReport(false);
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-32">
        <div className="flex items-center gap-3">
          <div className="w-3 h-3 border border-zinc-600 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm" style={{ color: "var(--muted)" }}>
            Loading search results...
          </span>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="mx-auto max-w-5xl px-6 py-16 text-center">
        <p className="text-sm" style={{ color: "var(--muted)" }}>
          Search not found.
        </p>
        <Link
          href="/search"
          className="text-xs mt-4 inline-block"
          style={{ color: "var(--accent)" }}
        >
          Back to searches
        </Link>
      </div>
    );
  }

  const { search, results } = data;
  const isSearching =
    search.status === "searching" || search.status === "analyzing" || search.status === "pending";

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <div className="flex items-center gap-2 mb-6">
        <Link
          href="/search"
          className="text-xs"
          style={{ color: "var(--muted)" }}
        >
          Prior Art Search
        </Link>
        <span className="text-xs" style={{ color: "var(--muted)" }}>
          /
        </span>
        <span className="text-xs" style={{ color: "var(--foreground)" }}>
          Results
        </span>
      </div>

      {/* Search summary */}
      <div
        className="p-5 rounded border mb-6"
        style={{ borderColor: "var(--border)", background: "var(--surface)" }}
      >
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <h1
              className="text-sm font-medium mb-2"
              style={{ color: "var(--foreground)" }}
            >
              {search.query_text}
            </h1>
            <div className="flex flex-wrap items-center gap-3">
              {search.ipc_classification && (
                <span className="text-xs px-2 py-0.5 rounded" style={{ background: "var(--surface-raised)", color: "var(--muted)" }}>
                  IPC: {search.ipc_classification}
                </span>
              )}
              {search.applicant_filter && (
                <span className="text-xs px-2 py-0.5 rounded" style={{ background: "var(--surface-raised)", color: "var(--muted)" }}>
                  Applicant: {search.applicant_filter}
                </span>
              )}
              {search.date_from && (
                <span className="text-xs px-2 py-0.5 rounded" style={{ background: "var(--surface-raised)", color: "var(--muted)" }}>
                  From: {search.date_from}
                </span>
              )}
              {search.date_to && (
                <span className="text-xs px-2 py-0.5 rounded" style={{ background: "var(--surface-raised)", color: "var(--muted)" }}>
                  To: {search.date_to}
                </span>
              )}
              {search.include_npl && (
                <span className="text-xs px-2 py-0.5 rounded" style={{ background: "var(--surface-raised)", color: "var(--muted)" }}>
                  +NPL
                </span>
              )}
            </div>
          </div>
          <div className="flex items-center gap-3">
            {search.status === "complete" && (
              <>
                {reportUrl ? (
                  <a
                    href={reportUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="px-3 py-1.5 rounded text-xs font-medium"
                    style={{ background: "var(--accent)", color: "var(--background)" }}
                  >
                    Download PDF
                  </a>
                ) : (
                  <button
                    onClick={handleGenerateReport}
                    disabled={generatingReport}
                    className="px-3 py-1.5 rounded text-xs font-medium transition-colors disabled:opacity-50"
                    style={{ background: "var(--accent)", color: "var(--background)" }}
                  >
                    {generatingReport ? "Generating..." : "Export PDF Report"}
                  </button>
                )}
              </>
            )}
            <StatusBadge status={search.status} />
          </div>
        </div>
      </div>

      {/* Loading state */}
      {isSearching && (
        <div className="flex flex-col items-center justify-center py-16">
          <div className="w-4 h-4 border-2 border-zinc-600 border-t-transparent rounded-full animate-spin mb-4" />
          <p className="text-sm" style={{ color: "var(--foreground)" }}>
            {search.status === "analyzing"
              ? "AI is analyzing results for novelty..."
              : "Searching patent databases..."}
          </p>
          <p className="text-xs mt-2" style={{ color: "var(--muted)" }}>
            This typically takes 1-3 minutes
          </p>
        </div>
      )}

      {/* Failed state */}
      {search.status === "failed" && (
        <div className="text-center py-16">
          <p className="text-sm" style={{ color: "#ef4444" }}>
            Search failed. Please try again.
          </p>
          <Link
            href="/search"
            className="text-xs mt-4 inline-block"
            style={{ color: "var(--accent)" }}
          >
            New search
          </Link>
        </div>
      )}

      {/* Results */}
      {search.status === "complete" && (
        <>
          <div className="flex items-center justify-between mb-4">
            <h2
              className="text-sm font-medium"
              style={{ color: "var(--foreground)" }}
            >
              {results.length} result{results.length !== 1 ? "s" : ""} found
            </h2>
            <div className="flex items-center gap-4">
              <SourceCount results={results} source="inpass" label="InPASS" />
              <SourceCount results={results} source="google_patents" label="Google Patents" />
              <SourceCount results={results} source="npl" label="NPL" />
            </div>
          </div>

          {results.length === 0 ? (
            <div className="text-center py-16">
              <p className="text-sm" style={{ color: "var(--muted)" }}>
                No prior art found matching your search criteria.
              </p>
              <p className="text-xs mt-2" style={{ color: "var(--muted)" }}>
                This may indicate good novelty prospects for your invention.
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {results.map((result) => (
                <ResultCard key={result.id} result={result} />
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const color =
    status === "complete"
      ? "var(--accent)"
      : status === "failed"
        ? "#ef4444"
        : "#f59e0b";

  return (
    <span
      className="text-xs font-medium px-2 py-0.5 rounded"
      style={{ color, background: `${color}15` }}
    >
      {status}
    </span>
  );
}

function SourceCount({
  results,
  source,
  label,
}: {
  results: PriorArtResult[];
  source: string;
  label: string;
}) {
  const count = results.filter((r) => r.source === source).length;
  if (count === 0) return null;
  return (
    <span className="text-xs" style={{ color: "var(--muted)" }}>
      {label}: {count}
    </span>
  );
}

export default function SearchResultsPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <SearchResultsContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
