"use client";

import { useState } from "react";
import { ObjectionWithResponse, api } from "@/lib/api-client";

interface ObjectionCardProps {
  objection: ObjectionWithResponse;
  streamingContent?: string;
  isGenerating?: boolean;
  onResponseUpdated?: () => void;
}

const CATEGORY_COLORS: Record<string, string> = {
  novelty: "#ef4444",
  inventive_step: "#f59e0b",
  non_patentable: "#8b5cf6",
  insufficiency: "#3b82f6",
  unity: "#06b6d4",
  formal: "#6b7280",
  other: "#6b7280",
};

const CATEGORY_LABELS: Record<string, string> = {
  novelty: "Novelty (Sec 2(1)(j))",
  inventive_step: "Inventive Step (Sec 2(1)(ja))",
  non_patentable: "Non-Patentable (Sec 3)",
  insufficiency: "Insufficiency (Sec 10)",
  unity: "Unity (Sec 10(5))",
  formal: "Formal",
  other: "Other",
};

export function ObjectionCard({
  objection,
  streamingContent,
  isGenerating,
  onResponseUpdated,
}: ObjectionCardProps) {
  const [expanded, setExpanded] = useState(true);
  const [editing, setEditing] = useState(false);
  const [editText, setEditText] = useState("");
  const [saving, setSaving] = useState(false);
  const [regenerating, setRegenerating] = useState(false);

  const response = objection.response;
  const color = CATEGORY_COLORS[objection.category] || "#6b7280";
  const label = CATEGORY_LABELS[objection.category] || objection.category;

  const hasResponse =
    response &&
    response.status !== "pending" &&
    (response.legal_arguments || response.claim_amendments || response.case_law_citations);

  async function handleSaveEdit() {
    if (!response) return;
    setSaving(true);
    try {
      await api.updateFerResponse(response.id, { user_edited_text: editText });
      setEditing(false);
      onResponseUpdated?.();
    } catch (err) {
      console.error("Failed to save edit:", err);
    } finally {
      setSaving(false);
    }
  }

  async function handleAccept() {
    if (!response) return;
    try {
      await api.acceptFerResponse(response.id);
      onResponseUpdated?.();
    } catch (err) {
      console.error("Failed to accept:", err);
    }
  }

  function startEdit() {
    if (!response) return;
    setEditText(
      response.user_edited_text ||
        [response.legal_arguments, response.claim_amendments, response.case_law_citations]
          .filter(Boolean)
          .join("\n\n---\n\n")
    );
    setEditing(true);
  }

  return (
    <div
      className="rounded border"
      style={{ borderColor: "var(--border)", background: "var(--surface)" }}
    >
      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full text-left p-4 flex items-start gap-3"
      >
        <div
          className="mt-0.5 w-5 h-5 rounded flex items-center justify-center flex-shrink-0 text-xs font-mono font-bold"
          style={{ background: `${color}20`, color }}
        >
          {objection.objection_number}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span
              className="text-xs font-medium px-1.5 py-0.5 rounded"
              style={{ background: `${color}15`, color }}
            >
              {label}
            </span>
            {objection.section_reference && (
              <span
                className="text-xs px-1.5 py-0.5 rounded"
                style={{
                  background: "var(--surface-raised)",
                  color: "var(--muted)",
                }}
              >
                {objection.section_reference}
              </span>
            )}
            {response && (
              <span
                className="text-xs px-1.5 py-0.5 rounded"
                style={{
                  background:
                    response.status === "accepted"
                      ? "var(--accent)"
                      : response.status === "complete" || response.status === "edited"
                        ? "#3b82f620"
                        : response.status === "generating"
                          ? "#f59e0b20"
                          : "var(--surface-raised)",
                  color:
                    response.status === "accepted"
                      ? "var(--background)"
                      : response.status === "complete" || response.status === "edited"
                        ? "#3b82f6"
                        : response.status === "generating"
                          ? "#f59e0b"
                          : "var(--muted)",
                }}
              >
                {response.status === "accepted"
                  ? "Accepted"
                  : response.status === "edited"
                    ? "Edited"
                    : response.status === "complete"
                      ? "Response ready"
                      : response.status === "generating"
                        ? "Generating..."
                        : "Pending"}
              </span>
            )}
          </div>
          <p
            className="text-sm"
            style={{ color: "var(--foreground)" }}
          >
            {objection.summary}
          </p>
        </div>
        <span className="text-xs" style={{ color: "var(--muted)" }}>
          {expanded ? "−" : "+"}
        </span>
      </button>

      {expanded && (
        <div className="px-4 pb-4 space-y-4">
          {/* Objection full text */}
          <div>
            <h4
              className="text-xs font-medium mb-2 uppercase tracking-wider"
              style={{ color: "var(--muted)" }}
            >
              Examiner&apos;s Objection
            </h4>
            <div
              className="text-xs p-3 rounded whitespace-pre-wrap"
              style={{
                background: "var(--background)",
                color: "var(--foreground)",
                borderLeft: `2px solid ${color}`,
              }}
            >
              {objection.full_text}
            </div>
          </div>

          {/* Streaming content */}
          {isGenerating && streamingContent && (
            <div>
              <h4
                className="text-xs font-medium mb-2 uppercase tracking-wider flex items-center gap-2"
                style={{ color: "#f59e0b" }}
              >
                <div className="w-2 h-2 border border-amber-500 border-t-transparent rounded-full animate-spin" />
                Generating Response...
              </h4>
              <div
                className="text-xs p-3 rounded whitespace-pre-wrap"
                style={{
                  background: "var(--background)",
                  color: "var(--foreground)",
                }}
              >
                {streamingContent}
              </div>
            </div>
          )}

          {/* Response sections */}
          {hasResponse && !editing && (
            <>
              {response.legal_arguments && (
                <ResponseSection
                  title="Legal Arguments"
                  content={response.legal_arguments}
                />
              )}
              {response.claim_amendments && (
                <ResponseSection
                  title="Suggested Claim Amendments"
                  content={response.claim_amendments}
                />
              )}
              {response.case_law_citations && (
                <ResponseSection
                  title="Indian Case Law & Citations"
                  content={response.case_law_citations}
                />
              )}

              {response.user_edited_text && (
                <ResponseSection
                  title="Your Edited Response"
                  content={response.user_edited_text}
                  highlight
                />
              )}

              {/* Action buttons */}
              <div className="flex items-center gap-2 pt-2">
                <button
                  onClick={startEdit}
                  className="px-3 py-1.5 rounded text-xs font-medium border transition-colors hover:border-zinc-500"
                  style={{
                    borderColor: "var(--border)",
                    color: "var(--foreground)",
                  }}
                >
                  Edit Response
                </button>
                {response.status !== "accepted" && (
                  <button
                    onClick={handleAccept}
                    className="px-3 py-1.5 rounded text-xs font-medium transition-colors"
                    style={{
                      background: "var(--accent)",
                      color: "var(--background)",
                    }}
                  >
                    Accept Response
                  </button>
                )}
              </div>
            </>
          )}

          {/* Edit mode */}
          {editing && (
            <div>
              <h4
                className="text-xs font-medium mb-2 uppercase tracking-wider"
                style={{ color: "var(--foreground)" }}
              >
                Edit Response
              </h4>
              <textarea
                value={editText}
                onChange={(e) => setEditText(e.target.value)}
                rows={12}
                className="w-full rounded border px-3 py-2 text-xs font-mono resize-y focus:outline-none"
                style={{
                  borderColor: "var(--border)",
                  background: "var(--background)",
                  color: "var(--foreground)",
                }}
              />
              <div className="flex items-center gap-2 mt-2">
                <button
                  onClick={handleSaveEdit}
                  disabled={saving}
                  className="px-3 py-1.5 rounded text-xs font-medium transition-colors disabled:opacity-50"
                  style={{
                    background: "var(--accent)",
                    color: "var(--background)",
                  }}
                >
                  {saving ? "Saving..." : "Save Edit"}
                </button>
                <button
                  onClick={() => setEditing(false)}
                  className="px-3 py-1.5 rounded text-xs font-medium border transition-colors"
                  style={{
                    borderColor: "var(--border)",
                    color: "var(--muted)",
                  }}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ResponseSection({
  title,
  content,
  highlight,
}: {
  title: string;
  content: string;
  highlight?: boolean;
}) {
  return (
    <div>
      <h4
        className="text-xs font-medium mb-2 uppercase tracking-wider"
        style={{ color: highlight ? "var(--accent)" : "var(--muted)" }}
      >
        {title}
      </h4>
      <div
        className="text-xs p-3 rounded whitespace-pre-wrap"
        style={{
          background: highlight ? "var(--accent)08" : "var(--background)",
          color: "var(--foreground)",
          borderLeft: highlight ? "2px solid var(--accent)" : undefined,
        }}
      >
        {content}
      </div>
    </div>
  );
}
