"use client";

import { useState } from "react";

interface FerUploadFormProps {
  onSubmit: (data: {
    fer_text: string;
    title?: string;
    application_number?: string;
    fer_date?: string;
  }) => void;
  submitting: boolean;
}

export function FerUploadForm({ onSubmit, submitting }: FerUploadFormProps) {
  const [ferText, setFerText] = useState("");
  const [showDetails, setShowDetails] = useState(false);
  const [title, setTitle] = useState("");
  const [applicationNumber, setApplicationNumber] = useState("");
  const [ferDate, setFerDate] = useState("");

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!ferText.trim()) return;

    onSubmit({
      fer_text: ferText,
      title: title || undefined,
      application_number: applicationNumber || undefined,
      fer_date: ferDate || undefined,
    });
  }

  return (
    <form onSubmit={handleSubmit}>
      <div
        className="p-5 rounded border"
        style={{ borderColor: "var(--border)", background: "var(--surface)" }}
      >
        <label
          className="block text-sm font-medium mb-2"
          style={{ color: "var(--foreground)" }}
        >
          FER Content
        </label>
        <textarea
          value={ferText}
          onChange={(e) => setFerText(e.target.value)}
          placeholder="Paste the full text of the First Examination Report here..."
          rows={10}
          className="w-full rounded border px-3 py-2 text-sm resize-y focus:outline-none"
          style={{
            borderColor: "var(--border)",
            background: "var(--background)",
            color: "var(--foreground)",
          }}
        />
        <p className="text-xs mt-1" style={{ color: "var(--muted)" }}>
          Paste the complete FER text including all objections raised by the
          examiner
        </p>

        <button
          type="button"
          onClick={() => setShowDetails(!showDetails)}
          className="text-xs mt-4 mb-2"
          style={{ color: "var(--accent)" }}
        >
          {showDetails ? "Hide details" : "Add optional details"}
        </button>

        {showDetails && (
          <div className="space-y-3 mt-2">
            <div>
              <label
                className="block text-xs mb-1"
                style={{ color: "var(--muted)" }}
              >
                Analysis Title
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="e.g., FER Response for Application 202011012345"
                className="w-full rounded border px-3 py-1.5 text-sm focus:outline-none"
                style={{
                  borderColor: "var(--border)",
                  background: "var(--background)",
                  color: "var(--foreground)",
                }}
              />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label
                  className="block text-xs mb-1"
                  style={{ color: "var(--muted)" }}
                >
                  Application Number
                </label>
                <input
                  type="text"
                  value={applicationNumber}
                  onChange={(e) => setApplicationNumber(e.target.value)}
                  placeholder="e.g., 202011012345"
                  className="w-full rounded border px-3 py-1.5 text-sm focus:outline-none"
                  style={{
                    borderColor: "var(--border)",
                    background: "var(--background)",
                    color: "var(--foreground)",
                  }}
                />
              </div>
              <div>
                <label
                  className="block text-xs mb-1"
                  style={{ color: "var(--muted)" }}
                >
                  FER Date
                </label>
                <input
                  type="date"
                  value={ferDate}
                  onChange={(e) => setFerDate(e.target.value)}
                  className="w-full rounded border px-3 py-1.5 text-sm focus:outline-none"
                  style={{
                    borderColor: "var(--border)",
                    background: "var(--background)",
                    color: "var(--foreground)",
                  }}
                />
              </div>
            </div>
          </div>
        )}

        <div className="mt-4">
          <button
            type="submit"
            disabled={submitting || !ferText.trim()}
            className="px-4 py-2 rounded text-sm font-medium transition-colors disabled:opacity-50"
            style={{
              background: "var(--accent)",
              color: "var(--background)",
            }}
          >
            {submitting ? "Analyzing FER..." : "Analyze FER"}
          </button>
        </div>
      </div>
    </form>
  );
}
