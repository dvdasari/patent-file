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
  const eventSourceRef = useRef<EventSource | null>(null);

  const startGeneration = useCallback((projectId: string) => {
    setState({
      sections: {},
      currentSection: null,
      totalSections: 0,
      completedSections: 0,
      isComplete: false,
      error: null,
    });

    const url = `${API_URL}/api/projects/${projectId}/generate`;
    const eventSource = new EventSource(url, { withCredentials: true });
    eventSourceRef.current = eventSource;

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        switch (data.event) {
          case "section_start":
            setState((prev) => ({
              ...prev,
              currentSection: data.data.section_type,
              totalSections: data.data.total,
              sections: {
                ...prev.sections,
                [data.data.section_type]: {
                  content: "",
                  isGenerating: true,
                  isComplete: false,
                },
              },
            }));
            break;

          case "content_delta":
            setState((prev) => ({
              ...prev,
              sections: {
                ...prev.sections,
                [data.data.section_type]: {
                  ...prev.sections[data.data.section_type],
                  content:
                    (prev.sections[data.data.section_type]?.content || "") +
                    data.data.delta,
                },
              },
            }));
            break;

          case "section_complete":
            setState((prev) => ({
              ...prev,
              completedSections: prev.completedSections + 1,
              sections: {
                ...prev.sections,
                [data.data.section_type]: {
                  content: data.data.content,
                  isGenerating: false,
                  isComplete: true,
                },
              },
            }));
            break;

          case "generation_complete":
            setState((prev) => ({
              ...prev,
              isComplete: true,
              currentSection: null,
            }));
            eventSource.close();
            break;

          case "error":
            setState((prev) => ({
              ...prev,
              error: data.data.message,
              currentSection: null,
            }));
            eventSource.close();
            break;
        }
      } catch {
        // Ignore parse errors
      }
    };

    eventSource.onerror = () => {
      setState((prev) => ({
        ...prev,
        error: "Connection lost. Please retry.",
      }));
      eventSource.close();
    };
  }, []);

  const stopGeneration = useCallback(() => {
    eventSourceRef.current?.close();
  }, []);

  return { ...state, startGeneration, stopGeneration };
}
