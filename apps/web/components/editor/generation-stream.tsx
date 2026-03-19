"use client";

import { useSSEStream } from "@/hooks/use-sse-stream";
import { useEffect } from "react";

interface GenerationStreamProps {
  projectId: string;
  onComplete: () => void;
}

const SECTION_LABELS: Record<string, string> = {
  title: "Title",
  field_of_invention: "Field of Invention",
  background: "Background",
  summary: "Summary",
  detailed_description: "Detailed Description",
  claims: "Claims",
  abstract: "Abstract",
  drawings_description: "Drawings Description",
};

export function GenerationStream({ projectId, onComplete }: GenerationStreamProps) {
  const stream = useSSEStream();

  useEffect(() => {
    stream.startGeneration(projectId);
    return () => stream.stopGeneration();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId]);

  useEffect(() => {
    if (stream.isComplete) onComplete();
  }, [stream.isComplete, onComplete]);

  const progress = stream.totalSections
    ? (stream.completedSections / stream.totalSections) * 100
    : 0;

  return (
    <div className="space-y-6 animate-fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-base font-medium text-zinc-100">Generating Patent Draft</h2>
          <p className="text-xs mt-1" style={{ color: "var(--muted)" }}>
            AI is drafting each section sequentially...
          </p>
        </div>
        <span className="text-sm font-mono" style={{ color: "var(--accent)" }}>
          {stream.completedSections}/{stream.totalSections || "?"}
        </span>
      </div>

      {/* Progress bar */}
      <div className="h-0.5 rounded-full" style={{ background: "var(--border)" }}>
        <div
          className="h-0.5 rounded-full transition-all duration-700 ease-out"
          style={{ width: `${progress}%`, background: "var(--accent)" }}
        />
      </div>

      {stream.error && (
        <div className="rounded border px-4 py-3 text-sm animate-fade-in"
          style={{ borderColor: "rgba(239, 68, 68, 0.3)", background: "rgba(239, 68, 68, 0.05)", color: "#f87171" }}>
          {stream.error}
        </div>
      )}

      <div className="space-y-2">
        {Object.entries(stream.sections).map(([type, section], i) => (
          <div
            key={type}
            className="rounded border p-4 animate-fade-in"
            style={{
              borderColor: section.isGenerating ? "var(--accent-dim)" : "var(--border)",
              background: "var(--surface)",
              animationDelay: `${i * 50}ms`,
            }}
          >
            <div className="flex items-center gap-2 mb-2">
              <h3 className="text-xs font-medium uppercase tracking-wider text-zinc-400">
                {SECTION_LABELS[type] || type}
              </h3>
              {section.isGenerating && (
                <div className="flex items-center gap-1.5">
                  <div className="w-1.5 h-1.5 rounded-full animate-pulse" style={{ background: "var(--accent)" }} />
                  <span className="text-xs" style={{ color: "var(--accent-dim)" }}>writing...</span>
                </div>
              )}
              {section.isComplete && (
                <span className="text-xs text-green-500">✓</span>
              )}
            </div>
            <pre className="whitespace-pre-wrap text-sm text-zinc-400 leading-relaxed font-[var(--font-geist-sans)]">
              {section.content.slice(0, 300)}{section.content.length > 300 ? "..." : ""}
            </pre>
          </div>
        ))}
      </div>
    </div>
  );
}
