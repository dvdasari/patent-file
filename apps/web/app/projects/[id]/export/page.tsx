"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { api } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";

interface ExportRecord {
  id: string;
  format: string;
  file_size_bytes: number;
  created_at: string;
}

function ExportContent() {
  const params = useParams();
  const projectId = params.id as string;
  const [exports, setExports] = useState<ExportRecord[]>([]);
  const [generating, setGenerating] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.listExports(projectId)
      .then((data) => setExports((data as unknown as ExportRecord[]) || []))
      .catch(() => {});
  }, [projectId]);

  async function handleExport(format: string) {
    setGenerating(format);
    setError(null);
    try {
      const result = (await api.createExport(projectId, format)) as unknown as ExportRecord;
      setExports((prev) => [result, ...prev]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Export failed");
    } finally {
      setGenerating(null);
    }
  }

  async function handleDownload(exportId: string) {
    try {
      const { url } = await api.getDownloadUrl(exportId);
      window.open(url, "_blank");
    } catch {
      setError("Failed to get download URL");
    }
  }

  return (
    <div className="mx-auto max-w-2xl px-6 py-10 animate-fade-in">
      <div className="mb-8">
        <h1 className="text-lg font-medium tracking-tight text-zinc-100">Export</h1>
        <p className="text-xs mt-1" style={{ color: "var(--muted)" }}>
          Generate IPO Form 2 compliant documents
        </p>
      </div>

      <div className="grid grid-cols-2 gap-3 mb-8">
        {[
          { format: "pdf", label: "PDF", desc: "IPO Form 2 layout" },
          { format: "docx", label: "DOCX", desc: "Editable Word format" },
        ].map(({ format, label, desc }) => (
          <button
            key={format}
            onClick={() => handleExport(format)}
            disabled={generating !== null}
            className="group rounded border p-6 text-left transition-all duration-200 disabled:opacity-40 hover:-translate-y-0.5"
            style={{ borderColor: "var(--border)", background: "var(--surface)" }}
            onMouseEnter={(e) => { e.currentTarget.style.borderColor = "var(--accent-dim)"; }}
            onMouseLeave={(e) => { e.currentTarget.style.borderColor = "var(--border)"; }}
          >
            <div className="text-lg font-mono font-medium text-zinc-200 mb-1">.{format}</div>
            <p className="text-xs" style={{ color: "var(--muted)" }}>{desc}</p>
            {generating === format && (
              <div className="mt-3 flex items-center gap-2">
                <div className="w-2.5 h-2.5 border border-zinc-500 border-t-transparent rounded-full animate-spin" />
                <span className="text-xs" style={{ color: "var(--accent)" }}>Generating...</span>
              </div>
            )}
          </button>
        ))}
      </div>

      {error && (
        <div className="mb-6 rounded border px-4 py-3 text-sm"
          style={{ borderColor: "rgba(239, 68, 68, 0.3)", background: "rgba(239, 68, 68, 0.05)", color: "#f87171" }}>
          {error}
        </div>
      )}

      {exports.length > 0 && (
        <div>
          <h2 className="text-xs font-medium uppercase tracking-wider mb-3" style={{ color: "var(--muted)" }}>
            Previous Exports
          </h2>
          <div className="space-y-1.5">
            {exports.map((exp) => (
              <div
                key={exp.id}
                className="flex items-center justify-between rounded border px-4 py-3"
                style={{ borderColor: "var(--border)", background: "var(--surface)" }}
              >
                <div className="flex items-center gap-3">
                  <span className="text-xs font-mono font-medium uppercase" style={{ color: "var(--accent)" }}>
                    {exp.format}
                  </span>
                  <span className="text-xs text-zinc-600">
                    {new Date(exp.created_at).toLocaleString("en-IN")} &middot; {(exp.file_size_bytes / 1024).toFixed(1)} KB
                  </span>
                </div>
                <button
                  onClick={() => handleDownload(exp.id)}
                  className="text-xs transition-colors"
                  style={{ color: "var(--accent-dim)" }}
                  onMouseEnter={(e) => { e.currentTarget.style.color = "var(--accent)"; }}
                  onMouseLeave={(e) => { e.currentTarget.style.color = "var(--accent-dim)"; }}
                >
                  Download ↓
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export default function ExportPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <ExportContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
