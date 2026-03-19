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

const SECTION_LABELS: Record<string, string> = {
  title: "TITLE OF THE INVENTION",
  field_of_invention: "FIELD OF THE INVENTION",
  background: "BACKGROUND OF THE INVENTION",
  summary: "SUMMARY OF THE INVENTION",
  detailed_description: "DETAILED DESCRIPTION",
  claims: "CLAIMS",
  abstract: "ABSTRACT",
  drawings_description: "BRIEF DESCRIPTION OF DRAWINGS",
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

  const isLong = content.split("\n").length > 5;
  const displayContent =
    !expanded && isLong
      ? content.split("\n").slice(0, 5).join("\n") + "\n..."
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

  function handleCancel() {
    setEditContent(content);
    setEditing(false);
  }

  return (
    <div className="rounded-lg border border-zinc-800 bg-zinc-900">
      <div className="flex items-center justify-between border-b border-zinc-800 px-4 py-3">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-medium text-zinc-200">
            {SECTION_LABELS[sectionType] || sectionType}
          </h3>
          <span className="inline-flex items-center rounded-full px-2 py-0.5 text-xs bg-zinc-800 text-zinc-400">
            {aiGenerated && editCount === 0
              ? "AI Generated"
              : `Edited (v${editCount})`}
          </span>
        </div>
        <div className="flex items-center gap-2">
          {!editing && (
            <button
              onClick={() => {
                setEditContent(content);
                setEditing(true);
              }}
              className="text-xs text-zinc-500 hover:text-zinc-300"
            >
              Edit
            </button>
          )}
        </div>
      </div>

      <div className="p-4">
        {editing ? (
          <div className="space-y-3">
            <textarea
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              rows={15}
              className="w-full rounded-md border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm text-zinc-100 font-mono focus:border-zinc-500 focus:outline-none"
            />
            <div className="flex items-center gap-2 justify-end">
              <button
                onClick={handleCancel}
                className="rounded-md border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800"
              >
                Cancel
              </button>
              <button
                onClick={handleSave}
                disabled={saving}
                className="rounded-md bg-zinc-100 px-3 py-1.5 text-xs font-medium text-zinc-900 hover:bg-zinc-200 disabled:opacity-50"
              >
                {saving ? "Saving..." : "Save"}
              </button>
            </div>
          </div>
        ) : (
          <div>
            <pre className="whitespace-pre-wrap text-sm text-zinc-300 font-mono leading-relaxed">
              {displayContent}
            </pre>
            {isLong && !expanded && (
              <button
                onClick={() => setExpanded(true)}
                className="mt-2 text-xs text-zinc-500 hover:text-zinc-300"
              >
                Show more
              </button>
            )}
            {expanded && isLong && (
              <button
                onClick={() => setExpanded(false)}
                className="mt-2 text-xs text-zinc-500 hover:text-zinc-300"
              >
                Show less
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
