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
    <div className="mx-auto max-w-2xl px-4 py-8">
      <h1 className="text-lg font-semibold text-zinc-100 mb-6">Export Patent Draft</h1>

      <div className="flex gap-4 mb-8">
        <button
          onClick={() => handleExport("pdf")}
          disabled={generating !== null}
          className="flex-1 rounded-lg border border-zinc-700 bg-zinc-900 p-6 text-center hover:border-zinc-500 disabled:opacity-50"
        >
          <div className="text-2xl mb-2">PDF</div>
          <p className="text-sm text-zinc-400">IPO Form 2 format</p>
          {generating === "pdf" && (
            <p className="mt-2 text-xs text-zinc-500 animate-pulse">Generating...</p>
          )}
        </button>
        <button
          onClick={() => handleExport("docx")}
          disabled={generating !== null}
          className="flex-1 rounded-lg border border-zinc-700 bg-zinc-900 p-6 text-center hover:border-zinc-500 disabled:opacity-50"
        >
          <div className="text-2xl mb-2">DOCX</div>
          <p className="text-sm text-zinc-400">Editable Word format</p>
          {generating === "docx" && (
            <p className="mt-2 text-xs text-zinc-500 animate-pulse">Generating...</p>
          )}
        </button>
      </div>

      {error && (
        <div className="mb-4 rounded-md bg-red-950/50 border border-red-900 px-3 py-2 text-sm text-red-400">
          {error}
        </div>
      )}

      {exports.length > 0 && (
        <div>
          <h2 className="text-sm font-medium text-zinc-300 mb-3">Previous Exports</h2>
          <div className="space-y-2">
            {exports.map((exp) => (
              <div
                key={exp.id}
                className="flex items-center justify-between rounded-lg border border-zinc-800 bg-zinc-900 px-4 py-3"
              >
                <div>
                  <span className="text-sm font-medium text-zinc-200 uppercase">
                    {exp.format}
                  </span>
                  <span className="ml-3 text-xs text-zinc-500">
                    {new Date(exp.created_at).toLocaleString()} &middot;{" "}
                    {(exp.file_size_bytes / 1024).toFixed(1)} KB
                  </span>
                </div>
                <button
                  onClick={() => handleDownload(exp.id)}
                  className="text-xs text-zinc-400 hover:text-zinc-100"
                >
                  Download
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
