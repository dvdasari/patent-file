"use client";

import { useState } from "react";
import { api } from "@/lib/api-client";

interface SectionCardProps {
  projectId: string;
  sectionType: string;
  content: string;
  aiGenerated: boolean;
  editCount: number;
  onContentUpdate: (newContent: string) => void;
}

const SECTION_LABELS: Record<string, { title: string; number: string }> = {
  title: { title: "TITLE OF THE INVENTION", number: "I" },
  field_of_invention: { title: "FIELD OF THE INVENTION", number: "II" },
  background: { title: "BACKGROUND OF THE INVENTION", number: "III" },
  summary: { title: "SUMMARY OF THE INVENTION", number: "IV" },
  detailed_description: { title: "DETAILED DESCRIPTION", number: "V" },
  claims: { title: "CLAIMS", number: "VI" },
  abstract: { title: "ABSTRACT", number: "VII" },
  drawings_description: { title: "BRIEF DESCRIPTION OF DRAWINGS", number: "VIII" },
};

export function SectionCard({
  projectId,
  sectionType,
  content,
  aiGenerated,
  editCount,
  onContentUpdate,
}: SectionCardProps) {
  const [editing, setEditing] = useState(false);
  const [editContent, setEditContent] = useState(content);
  const [saving, setSaving] = useState(false);
  const [expanded, setExpanded] = useState(false);

  const label = SECTION_LABELS[sectionType] || { title: sectionType, number: "?" };
  const isLong = content.split("\n").length > 8;
  const displayContent = !expanded && isLong
    ? content.split("\n").slice(0, 8).join("\n") + "\n..."
    : content;

  async function handleSave() {
    setSaving(true);
    try {
      await api.updateSection(projectId, sectionType, editContent);
      onContentUpdate(editContent);
      setEditing(false);
    } finally {
      setSaving(false);
    }
  }

  return (
    <div
      className="rounded border transition-all duration-200 group"
      style={{ borderColor: "var(--border)", background: "var(--surface)" }}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-5 py-3 border-b" style={{ borderColor: "var(--border-subtle)" }}>
        <div className="flex items-center gap-3">
          <span className="text-xs font-mono w-8" style={{ color: "var(--accent-dim)" }}>
            {label.number}
          </span>
          <h3 className="text-xs font-medium uppercase tracking-wider text-zinc-300">
            {label.title}
          </h3>
        </div>
        <div className="flex items-center gap-3">
          <span
            className="text-xs font-mono px-2 py-0.5 rounded"
            style={{
              color: aiGenerated && editCount === 0 ? "var(--accent)" : "var(--muted)",
              background: aiGenerated && editCount === 0 ? "rgba(200, 169, 110, 0.08)" : "var(--surface-raised)",
            }}
          >
            {aiGenerated && editCount === 0 ? "AI" : `v${editCount}`}
          </span>
          {!editing && (
            <button
              onClick={() => { setEditContent(content); setEditing(true); }}
              className="text-xs px-2 py-1 rounded transition-all opacity-0 group-hover:opacity-100"
              style={{ color: "var(--muted)", background: "var(--surface-raised)" }}
            >
              Edit
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="px-5 py-4">
        {editing ? (
          <div className="space-y-3 animate-fade-in">
            <textarea
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              rows={Math.min(20, editContent.split("\n").length + 3)}
              className="w-full rounded border bg-transparent px-3 py-3 text-sm leading-relaxed font-mono text-zinc-200 focus:outline-none transition-colors"
              style={{ borderColor: "var(--accent-dim)" }}
            />
            <div className="flex items-center gap-2 justify-end">
              <button
                onClick={() => { setEditContent(content); setEditing(false); }}
                className="rounded border px-3 py-1.5 text-xs transition-all"
                style={{ borderColor: "var(--border)", color: "var(--muted)" }}
              >
                Cancel
              </button>
              <button
                onClick={handleSave}
                disabled={saving}
                className="rounded px-4 py-1.5 text-xs font-medium transition-all disabled:opacity-40"
                style={{ background: "var(--accent)", color: "var(--background)" }}
              >
                {saving ? "Saving..." : "Save"}
              </button>
            </div>
          </div>
        ) : (
          <div>
            <pre className="whitespace-pre-wrap text-sm leading-relaxed text-zinc-300 font-[var(--font-geist-sans)]">
              {displayContent}
            </pre>
            {isLong && (
              <button
                onClick={() => setExpanded(!expanded)}
                className="mt-3 text-xs font-medium transition-colors"
                style={{ color: "var(--accent-dim)" }}
                onMouseEnter={(e) => { e.currentTarget.style.color = "var(--accent)"; }}
                onMouseLeave={(e) => { e.currentTarget.style.color = "var(--accent-dim)"; }}
              >
                {expanded ? "Show less ↑" : "Show more ↓"}
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
