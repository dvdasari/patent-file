"use client";

import { useState } from "react";

interface SearchFormProps {
  onSubmit: (params: {
    query: string;
    ipc_classification?: string;
    applicant?: string;
    date_from?: string;
    date_to?: string;
    include_npl?: boolean;
  }) => void;
  submitting: boolean;
}

export function SearchForm({ onSubmit, submitting }: SearchFormProps) {
  const [query, setQuery] = useState("");
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [ipc, setIpc] = useState("");
  const [applicant, setApplicant] = useState("");
  const [dateFrom, setDateFrom] = useState("");
  const [dateTo, setDateTo] = useState("");
  const [includeNpl, setIncludeNpl] = useState(false);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!query.trim()) return;

    onSubmit({
      query: query.trim(),
      ipc_classification: ipc.trim() || undefined,
      applicant: applicant.trim() || undefined,
      date_from: dateFrom || undefined,
      date_to: dateTo || undefined,
      include_npl: includeNpl,
    });
  }

  return (
    <form onSubmit={handleSubmit}>
      <div
        className="p-5 rounded border"
        style={{ borderColor: "var(--border)", background: "var(--surface)" }}
      >
        <label
          className="block text-xs font-medium mb-2"
          style={{ color: "var(--foreground)" }}
        >
          Invention Description
        </label>
        <textarea
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Describe the invention to search for prior art (e.g., 'solar-powered water purification system using UV-C LEDs with automatic flow control')"
          rows={3}
          className="w-full rounded border px-3 py-2 text-sm resize-none focus:outline-none focus:ring-1"
          style={{
            borderColor: "var(--border)",
            background: "var(--background)",
            color: "var(--foreground)",
          }}
        />

        <button
          type="button"
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="mt-3 text-xs transition-colors"
          style={{ color: "var(--accent)" }}
        >
          {showAdvanced ? "Hide" : "Show"} advanced filters
        </button>

        {showAdvanced && (
          <div className="mt-3 grid grid-cols-2 gap-4">
            <div>
              <label
                className="block text-xs mb-1"
                style={{ color: "var(--muted)" }}
              >
                IPC Classification
              </label>
              <input
                type="text"
                value={ipc}
                onChange={(e) => setIpc(e.target.value)}
                placeholder="e.g., A61K, C02F"
                className="w-full rounded border px-3 py-1.5 text-xs focus:outline-none focus:ring-1"
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
                Applicant / Assignee
              </label>
              <input
                type="text"
                value={applicant}
                onChange={(e) => setApplicant(e.target.value)}
                placeholder="e.g., CSIR, Tata"
                className="w-full rounded border px-3 py-1.5 text-xs focus:outline-none focus:ring-1"
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
                Filing Date From
              </label>
              <input
                type="date"
                value={dateFrom}
                onChange={(e) => setDateFrom(e.target.value)}
                className="w-full rounded border px-3 py-1.5 text-xs focus:outline-none focus:ring-1"
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
                Filing Date To
              </label>
              <input
                type="date"
                value={dateTo}
                onChange={(e) => setDateTo(e.target.value)}
                className="w-full rounded border px-3 py-1.5 text-xs focus:outline-none focus:ring-1"
                style={{
                  borderColor: "var(--border)",
                  background: "var(--background)",
                  color: "var(--foreground)",
                }}
              />
            </div>
            <div className="col-span-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={includeNpl}
                  onChange={(e) => setIncludeNpl(e.target.checked)}
                  className="rounded border"
                  style={{ borderColor: "var(--border)" }}
                />
                <span className="text-xs" style={{ color: "var(--foreground)" }}>
                  Include non-patent literature (CSIR, Indian scientific
                  publications)
                </span>
              </label>
            </div>
          </div>
        )}

        <div className="mt-4 flex justify-end">
          <button
            type="submit"
            disabled={submitting || !query.trim()}
            className="px-4 py-2 rounded text-xs font-medium transition-colors disabled:opacity-50"
            style={{
              background: "var(--accent)",
              color: "var(--background)",
            }}
          >
            {submitting ? "Searching..." : "Search Prior Art"}
          </button>
        </div>
      </div>
    </form>
  );
}
