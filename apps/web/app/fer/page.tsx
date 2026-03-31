"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { api, FerAnalysis } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { FerUploadForm } from "@/components/fer/fer-upload-form";

function FerContent() {
  const router = useRouter();
  const [analyses, setAnalyses] = useState<FerAnalysis[]>([]);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    api
      .listFer()
      .then((data) => {
        setAnalyses(data || []);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  async function handleSubmit(params: {
    fer_text: string;
    title?: string;
    application_number?: string;
    fer_date?: string;
  }) {
    setSubmitting(true);
    try {
      const res = await api.createFer(params);
      router.push(`/fer/${res.id}`);
    } catch (err) {
      console.error("FER creation failed:", err);
      setSubmitting(false);
    }
  }

  const statusColor = (status: string) => {
    switch (status) {
      case "complete":
        return "var(--accent)";
      case "parsing":
      case "generating":
        return "#f59e0b";
      case "parsed":
        return "#3b82f6";
      case "failed":
        return "#ef4444";
      default:
        return "var(--muted)";
    }
  };

  const statusLabel = (status: string) => {
    switch (status) {
      case "parsing":
        return "Parsing FER...";
      case "parsed":
        return "Ready to generate";
      case "generating":
        return "Generating responses...";
      case "complete":
        return "Complete";
      case "failed":
        return "Failed";
      default:
        return status;
    }
  };

  const categoryLabel = (cat: string) => {
    const map: Record<string, string> = {
      novelty: "Novelty",
      inventive_step: "Inventive Step",
      non_patentable: "Non-Patentable",
      insufficiency: "Insufficiency",
      unity: "Unity",
      formal: "Formal",
      other: "Other",
    };
    return map[cat] || cat;
  };

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <div className="mb-8">
        <h1
          className="text-lg font-medium mb-1"
          style={{ color: "var(--foreground)" }}
        >
          FER Response Assistant
        </h1>
        <p className="text-xs" style={{ color: "var(--muted)" }}>
          Upload a First Examination Report and generate AI-powered responses to
          examiner objections
        </p>
      </div>

      <FerUploadForm onSubmit={handleSubmit} submitting={submitting} />

      <div className="mt-10">
        <h2
          className="text-sm font-medium mb-4"
          style={{ color: "var(--foreground)" }}
        >
          Recent Analyses
        </h2>

        {loading ? (
          <div className="flex items-center gap-3 py-8">
            <div className="w-3 h-3 border border-zinc-600 border-t-transparent rounded-full animate-spin" />
            <span className="text-xs" style={{ color: "var(--muted)" }}>
              Loading analyses...
            </span>
          </div>
        ) : analyses.length === 0 ? (
          <p className="text-xs py-8" style={{ color: "var(--muted)" }}>
            No FER analyses yet. Upload your first FER above to get started.
          </p>
        ) : (
          <div className="space-y-2">
            {analyses.map((a) => (
              <button
                key={a.id}
                onClick={() => router.push(`/fer/${a.id}`)}
                className="w-full text-left p-4 rounded border transition-colors hover:border-zinc-600"
                style={{
                  borderColor: "var(--border)",
                  background: "var(--surface)",
                }}
              >
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <p
                      className="text-sm font-medium truncate"
                      style={{ color: "var(--foreground)" }}
                    >
                      {a.title}
                    </p>
                    <div className="flex items-center gap-3 mt-1">
                      <span
                        className="text-xs"
                        style={{ color: "var(--muted)" }}
                      >
                        {new Date(a.created_at).toLocaleDateString()}
                      </span>
                      {a.application_number && (
                        <span
                          className="text-xs"
                          style={{ color: "var(--muted)" }}
                        >
                          App: {a.application_number}
                        </span>
                      )}
                      {a.examiner_name && (
                        <span
                          className="text-xs"
                          style={{ color: "var(--muted)" }}
                        >
                          Examiner: {a.examiner_name}
                        </span>
                      )}
                    </div>
                  </div>
                  <span
                    className="text-xs font-medium px-2 py-0.5 rounded"
                    style={{
                      color: statusColor(a.status),
                      background: `${statusColor(a.status)}15`,
                    }}
                  >
                    {statusLabel(a.status)}
                  </span>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default function FerPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <FerContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
