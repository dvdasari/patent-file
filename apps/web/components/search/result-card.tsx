"use client";

import { useState } from "react";
import { PriorArtResult } from "@/lib/api-client";

interface ResultCardProps {
  result: PriorArtResult;
}

export function ResultCard({ result }: ResultCardProps) {
  const [expanded, setExpanded] = useState(false);
  const scorePct = Math.round(result.similarity_score * 100);

  const scoreColor =
    scorePct >= 70 ? "#ef4444" : scorePct >= 40 ? "#f59e0b" : "var(--accent)";

  const sourceLabel: Record<string, string> = {
    inpass: "InPASS",
    google_patents: "Google Patents",
    npl: "NPL",
    csir: "CSIR",
  };

  return (
    <div
      className="rounded border overflow-hidden"
      style={{ borderColor: "var(--border)", background: "var(--surface)" }}
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full text-left p-4"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span
                className="text-xs font-mono px-1.5 py-0.5 rounded"
                style={{
                  background: "var(--surface-raised)",
                  color: "var(--muted)",
                }}
              >
                #{result.relevance_rank}
              </span>
              <span
                className="text-xs px-1.5 py-0.5 rounded"
                style={{
                  background: "var(--surface-raised)",
                  color: "var(--muted)",
                }}
              >
                {sourceLabel[result.source] || result.source}
              </span>
              {result.external_id && (
                <span
                  className="text-xs font-mono"
                  style={{ color: "var(--muted)" }}
                >
                  {result.external_id}
                </span>
              )}
            </div>
            <p
              className="text-sm font-medium"
              style={{ color: "var(--foreground)" }}
            >
              {result.title}
            </p>
            <div className="flex items-center gap-3 mt-1">
              {result.applicant && (
                <span className="text-xs" style={{ color: "var(--muted)" }}>
                  {result.applicant}
                </span>
              )}
              {result.filing_date && (
                <span className="text-xs" style={{ color: "var(--muted)" }}>
                  Filed: {result.filing_date}
                </span>
              )}
              {result.ipc_codes && (
                <span className="text-xs" style={{ color: "var(--muted)" }}>
                  IPC: {result.ipc_codes}
                </span>
              )}
            </div>
          </div>
          <div className="flex flex-col items-end gap-1 flex-shrink-0">
            <div
              className="text-lg font-mono font-bold"
              style={{ color: scoreColor }}
            >
              {scorePct}%
            </div>
            <span className="text-xs" style={{ color: "var(--muted)" }}>
              similarity
            </span>
          </div>
        </div>
      </button>

      {expanded && (
        <div
          className="px-4 pb-4 border-t"
          style={{ borderColor: "var(--border)" }}
        >
          {result.novelty_assessment && (
            <div className="mt-3">
              <h4
                className="text-xs font-medium mb-1"
                style={{ color: "var(--foreground)" }}
              >
                Novelty Assessment
              </h4>
              <p className="text-xs leading-relaxed" style={{ color: "var(--muted)" }}>
                {result.novelty_assessment}
              </p>
            </div>
          )}

          {result.abstract_text && (
            <div className="mt-3">
              <h4
                className="text-xs font-medium mb-1"
                style={{ color: "var(--foreground)" }}
              >
                Abstract
              </h4>
              <p className="text-xs leading-relaxed" style={{ color: "var(--muted)" }}>
                {result.abstract_text.length > 500
                  ? `${result.abstract_text.slice(0, 500)}...`
                  : result.abstract_text}
              </p>
            </div>
          )}

          <div className="mt-3 flex items-center gap-3">
            {result.publication_date && (
              <span className="text-xs" style={{ color: "var(--muted)" }}>
                Published: {result.publication_date}
              </span>
            )}
            {result.url && (
              <a
                href={result.url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs"
                style={{ color: "var(--accent)" }}
              >
                View source
              </a>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
