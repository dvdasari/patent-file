"use client";

import { useCallback, useRef, useState } from "react";

interface SectionState {
  content: string;
  isGenerating: boolean;
  isComplete: boolean;
}

interface StreamState {
  sections: Record<string, SectionState>;
  currentSection: string | null;
  totalSections: number;
  completedSections: number;
  isComplete: boolean;
  error: string | null;
}

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:5012";

export function useSSEStream() {
  const [state, setState] = useState<StreamState>({
    sections: {},
    currentSection: null,
    totalSections: 0,
    completedSections: 0,
    isComplete: false,
    error: null,
  });
  const abortRef = useRef<AbortController | null>(null);

  const startGeneration = useCallback((projectId: string) => {
    setState({
      sections: {},
      currentSection: null,
      totalSections: 0,
      completedSections: 0,
      isComplete: false,
      error: null,
    });

    const controller = new AbortController();
    abortRef.current = controller;

    const url = `${API_URL}/api/projects/${projectId}/generate`;

    // Use fetch with streaming instead of EventSource (EventSource can't send cookies cross-origin)
    fetch(url, {
      method: "POST",
      credentials: "include",
      headers: { "Accept": "text/event-stream" },
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          const body = await response.text().catch(() => "");
          setState((prev) => ({
            ...prev,
            error: `Generation failed: ${response.status} ${body}`,
          }));
          return;
        }

        const reader = response.body?.getReader();
        if (!reader) {
          setState((prev) => ({ ...prev, error: "No response stream" }));
          return;
        }

        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });

          // Process complete SSE lines (data: ...\n\n)
          const lines = buffer.split("\n");
          buffer = lines.pop() || ""; // Keep incomplete line in buffer

          for (const line of lines) {
            if (!line.startsWith("data: ")) continue;
            const jsonStr = line.slice(6).trim();
            if (!jsonStr) continue;

            try {
              const data = JSON.parse(jsonStr);

              if (data.section_start !== undefined || data.event === "section_start") {
                const d = data.section_start || data.data || data;
                setState((prev) => ({
                  ...prev,
                  currentSection: d.section_type,
                  totalSections: d.total || prev.totalSections,
                  sections: {
                    ...prev.sections,
                    [d.section_type]: { content: "", isGenerating: true, isComplete: false },
                  },
                }));
              } else if (data.content_delta !== undefined || data.event === "content_delta") {
                const d = data.content_delta || data.data || data;
                setState((prev) => ({
                  ...prev,
                  sections: {
                    ...prev.sections,
                    [d.section_type]: {
                      ...prev.sections[d.section_type],
                      content: (prev.sections[d.section_type]?.content || "") + d.delta,
                    },
                  },
                }));
              } else if (data.section_complete !== undefined || data.event === "section_complete") {
                const d = data.section_complete || data.data || data;
                setState((prev) => ({
                  ...prev,
                  completedSections: prev.completedSections + 1,
                  sections: {
                    ...prev.sections,
                    [d.section_type]: { content: d.content, isGenerating: false, isComplete: true },
                  },
                }));
              } else if (data.generation_complete !== undefined || data.event === "generation_complete") {
                setState((prev) => ({ ...prev, isComplete: true, currentSection: null }));
              } else if (data.error !== undefined || data.event === "error") {
                const d = data.error || data.data || data;
                setState((prev) => ({ ...prev, error: d.message || "Generation error" }));
              }
            } catch {
              // Skip unparseable lines
            }
          }
        }

        // If we get here without generation_complete, mark as complete anyway
        setState((prev) => {
          if (!prev.isComplete && !prev.error) {
            return { ...prev, isComplete: true };
          }
          return prev;
        });
      })
      .catch((err) => {
        if (err.name === "AbortError") return;
        setState((prev) => ({
          ...prev,
          error: `Connection failed: ${err.message}`,
        }));
      });
  }, []);

  const stopGeneration = useCallback(() => {
    abortRef.current?.abort();
  }, []);

  return { ...state, startGeneration, stopGeneration };
}
