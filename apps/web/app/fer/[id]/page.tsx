"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { useParams } from "next/navigation";
import { api, FerAnalysisDetail } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { ObjectionCard } from "@/components/fer/objection-card";
import Link from "next/link";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:5012";

function FerDetailContent() {
  const params = useParams();
  const analysisId = params.id as string;
  const [data, setData] = useState<FerAnalysisDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);
  const [streamingContent, setStreamingContent] = useState<
    Record<string, string>
  >({});
  const [activeObjection, setActiveObjection] = useState<string | null>(null);
  const [generationProgress, setGenerationProgress] = useState({
    current: 0,
    total: 0,
  });
  const abortRef = useRef<AbortController | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const res = await api.getFer(analysisId);
      setData(res);
      setLoading(false);
    } catch {
      setLoading(false);
    }
  }, [analysisId]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // Poll while parsing or generating
  useEffect(() => {
    if (!data) return;
    if (
      data.status !== "parsing" &&
      data.status !== "generating"
    )
      return;
    // Don't poll if we're handling SSE ourselves
    if (generating) return;

    const interval = setInterval(fetchData, 3000);
    return () => clearInterval(interval);
  }, [data, fetchData, generating]);

  function startGeneration() {
    setGenerating(true);
    setStreamingContent({});
    setActiveObjection(null);
    setGenerationProgress({ current: 0, total: 0 });

    const controller = new AbortController();
    abortRef.current = controller;

    fetch(`${API_URL}/api/fer/${analysisId}/generate`, {
      method: "POST",
      credentials: "include",
      headers: { Accept: "text/event-stream" },
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          setGenerating(false);
          return;
        }

        const reader = response.body?.getReader();
        if (!reader) {
          setGenerating(false);
          return;
        }

        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() || "";

          for (const line of lines) {
            if (!line.startsWith("data: ")) continue;
            const jsonStr = line.slice(6).trim();
            if (!jsonStr) continue;

            try {
              const evt = JSON.parse(jsonStr);

              if (evt.event === "objection_start") {
                const d = evt.data;
                setActiveObjection(d.objection_id);
                setGenerationProgress({ current: d.index + 1, total: d.total });
                setStreamingContent((prev) => ({
                  ...prev,
                  [d.objection_id]: "",
                }));
              } else if (evt.event === "content_delta") {
                const d = evt.data;
                setStreamingContent((prev) => ({
                  ...prev,
                  [d.objection_id]:
                    (prev[d.objection_id] || "") + d.delta,
                }));
              } else if (evt.event === "objection_complete") {
                const d = evt.data;
                setActiveObjection(null);
                // Clear streaming content for this objection
                setStreamingContent((prev) => {
                  const next = { ...prev };
                  delete next[d.objection_id];
                  return next;
                });
              } else if (evt.event === "generation_complete") {
                setGenerating(false);
                // Refresh data to get final state
                fetchData();
              } else if (evt.event === "error") {
                setGenerating(false);
                fetchData();
              }
            } catch {
              // Skip unparseable
            }
          }
        }

        setGenerating(false);
        fetchData();
      })
      .catch((err) => {
        if (err.name === "AbortError") return;
        setGenerating(false);
      });
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-32">
        <div className="flex items-center gap-3">
          <div className="w-3 h-3 border border-zinc-600 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm" style={{ color: "var(--muted)" }}>
            Loading FER analysis...
          </span>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="mx-auto max-w-5xl px-6 py-16 text-center">
        <p className="text-sm" style={{ color: "var(--muted)" }}>
          Analysis not found.
        </p>
        <Link
          href="/fer"
          className="text-xs mt-4 inline-block"
          style={{ color: "var(--accent)" }}
        >
          Back to FER analyses
        </Link>
      </div>
    );
  }

  const isParsing = data.status === "parsing";
  const isParsed = data.status === "parsed";
  const isComplete = data.status === "complete";

  return (
    <div className="mx-auto max-w-6xl px-6 py-8">
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 mb-6">
        <Link
          href="/fer"
          className="text-xs"
          style={{ color: "var(--muted)" }}
        >
          FER Assistant
        </Link>
        <span className="text-xs" style={{ color: "var(--muted)" }}>
          /
        </span>
        <span className="text-xs" style={{ color: "var(--foreground)" }}>
          {data.title}
        </span>
      </div>

      {/* Analysis header */}
      <div
        className="p-5 rounded border mb-6"
        style={{ borderColor: "var(--border)", background: "var(--surface)" }}
      >
        <div className="flex items-start justify-between">
          <div>
            <h1
              className="text-sm font-medium mb-2"
              style={{ color: "var(--foreground)" }}
            >
              {data.title}
            </h1>
            <div className="flex flex-wrap items-center gap-3">
              {data.application_number && (
                <span
                  className="text-xs px-2 py-0.5 rounded"
                  style={{
                    background: "var(--surface-raised)",
                    color: "var(--muted)",
                  }}
                >
                  App: {data.application_number}
                </span>
              )}
              {data.fer_date && (
                <span
                  className="text-xs px-2 py-0.5 rounded"
                  style={{
                    background: "var(--surface-raised)",
                    color: "var(--muted)",
                  }}
                >
                  FER Date: {data.fer_date}
                </span>
              )}
              {data.examiner_name && (
                <span
                  className="text-xs px-2 py-0.5 rounded"
                  style={{
                    background: "var(--surface-raised)",
                    color: "var(--muted)",
                  }}
                >
                  Examiner: {data.examiner_name}
                </span>
              )}
              <span
                className="text-xs px-2 py-0.5 rounded"
                style={{
                  background: "var(--surface-raised)",
                  color: "var(--muted)",
                }}
              >
                {data.objections.length} objection
                {data.objections.length !== 1 ? "s" : ""}
              </span>
            </div>
          </div>
          <div className="flex items-center gap-3">
            {(isParsed || isComplete) && (
              <button
                onClick={startGeneration}
                disabled={generating}
                className="px-3 py-1.5 rounded text-xs font-medium transition-colors disabled:opacity-50"
                style={{
                  background: "var(--accent)",
                  color: "var(--background)",
                }}
              >
                {generating
                  ? `Generating (${generationProgress.current}/${generationProgress.total})...`
                  : isComplete
                    ? "Regenerate All Responses"
                    : "Generate Responses"}
              </button>
            )}
            <StatusBadge status={data.status} />
          </div>
        </div>
      </div>

      {/* Parsing state */}
      {isParsing && (
        <div className="flex flex-col items-center justify-center py-16">
          <div className="w-4 h-4 border-2 border-zinc-600 border-t-transparent rounded-full animate-spin mb-4" />
          <p className="text-sm" style={{ color: "var(--foreground)" }}>
            AI is parsing the FER and extracting objections...
          </p>
          <p className="text-xs mt-2" style={{ color: "var(--muted)" }}>
            This typically takes 30-60 seconds
          </p>
        </div>
      )}

      {/* Failed state */}
      {data.status === "failed" && (
        <div className="text-center py-16">
          <p className="text-sm" style={{ color: "#ef4444" }}>
            FER parsing failed. Please try again with a clearer FER text.
          </p>
          <Link
            href="/fer"
            className="text-xs mt-4 inline-block"
            style={{ color: "var(--accent)" }}
          >
            New analysis
          </Link>
        </div>
      )}

      {/* Objections list (side-by-side view) */}
      {(isParsed || isComplete || generating) &&
        data.objections.length > 0 && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2
                className="text-sm font-medium"
                style={{ color: "var(--foreground)" }}
              >
                Objections & Responses
              </h2>
              {isComplete && (
                <div className="flex items-center gap-3">
                  <CategoryCount
                    objections={data.objections}
                    category="novelty"
                    label="Novelty"
                    color="#ef4444"
                  />
                  <CategoryCount
                    objections={data.objections}
                    category="inventive_step"
                    label="Inventive Step"
                    color="#f59e0b"
                  />
                  <CategoryCount
                    objections={data.objections}
                    category="non_patentable"
                    label="Sec 3"
                    color="#8b5cf6"
                  />
                  <CategoryCount
                    objections={data.objections}
                    category="insufficiency"
                    label="Insufficiency"
                    color="#3b82f6"
                  />
                </div>
              )}
            </div>

            <div className="space-y-3">
              {data.objections.map((obj) => (
                <ObjectionCard
                  key={obj.id}
                  objection={obj}
                  streamingContent={streamingContent[obj.id]}
                  isGenerating={activeObjection === obj.id}
                  onResponseUpdated={fetchData}
                />
              ))}
            </div>
          </div>
        )}

      {(isParsed || isComplete) && data.objections.length === 0 && (
        <div className="text-center py-16">
          <p className="text-sm" style={{ color: "var(--muted)" }}>
            No objections were found in this FER.
          </p>
        </div>
      )}
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, { color: string; label: string }> = {
    pending: { color: "var(--muted)", label: "Pending" },
    parsing: { color: "#f59e0b", label: "Parsing" },
    parsed: { color: "#3b82f6", label: "Parsed" },
    generating: { color: "#f59e0b", label: "Generating" },
    complete: { color: "var(--accent)", label: "Complete" },
    failed: { color: "#ef4444", label: "Failed" },
  };

  const { color, label } = map[status] || {
    color: "var(--muted)",
    label: status,
  };

  return (
    <span
      className="text-xs font-medium px-2 py-0.5 rounded"
      style={{ color, background: `${color}15` }}
    >
      {label}
    </span>
  );
}

function CategoryCount({
  objections,
  category,
  label,
  color,
}: {
  objections: { category: string }[];
  category: string;
  label: string;
  color: string;
}) {
  const count = objections.filter((o) => o.category === category).length;
  if (count === 0) return null;
  return (
    <span className="text-xs" style={{ color }}>
      {label}: {count}
    </span>
  );
}

export default function FerDetailPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <FerDetailContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
