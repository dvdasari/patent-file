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
    if (stream.isComplete) {
      onComplete();
    }
  }, [stream.isComplete, onComplete]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold text-zinc-100">
          Generating Patent Draft...
        </h2>
        <span className="text-sm text-zinc-400">
          {stream.completedSections} / {stream.totalSections || "?"} sections
        </span>
      </div>

      {/* Progress bar */}
      <div className="h-1 rounded-full bg-zinc-800">
        <div
          className="h-1 rounded-full bg-zinc-400 transition-all"
          style={{
            width: stream.totalSections
              ? `${(stream.completedSections / stream.totalSections) * 100}%`
              : "0%",
          }}
        />
      </div>

      {stream.error && (
        <div className="rounded-md bg-red-950/50 border border-red-900 px-3 py-2 text-sm text-red-400">
          {stream.error}
        </div>
      )}

      {/* Show completed + streaming sections */}
      <div className="space-y-3">
        {Object.entries(stream.sections).map(([type, section]) => (
          <div key={type} className="rounded-lg border border-zinc-800 bg-zinc-900 p-4">
            <div className="flex items-center gap-2 mb-2">
              <h3 className="text-sm font-medium text-zinc-200">
                {SECTION_LABELS[type] || type}
              </h3>
              {section.isGenerating && (
                <span className="text-xs text-zinc-500 animate-pulse">generating...</span>
              )}
              {section.isComplete && (
                <span className="text-xs text-green-400">done</span>
              )}
            </div>
            <pre className="whitespace-pre-wrap text-sm text-zinc-400 font-mono">
              {section.content || "..."}
            </pre>
          </div>
        ))}
      </div>
    </div>
  );
}
