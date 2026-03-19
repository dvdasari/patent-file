"use client";

import Link from "next/link";

interface ProjectCardProps {
  id: string;
  title: string;
  status: string;
  patent_type: string;
  updated_at: string;
}

const STATUS_CONFIG: Record<string, { label: string; color: string; bg: string }> = {
  draft: { label: "Draft", color: "#71717a", bg: "rgba(113, 113, 122, 0.1)" },
  interview_complete: { label: "Ready", color: "#c8a96e", bg: "rgba(200, 169, 110, 0.1)" },
  generating: { label: "Generating", color: "#60a5fa", bg: "rgba(96, 165, 250, 0.1)" },
  review: { label: "In Review", color: "#4ade80", bg: "rgba(74, 222, 128, 0.1)" },
  exported: { label: "Exported", color: "#a78bfa", bg: "rgba(167, 139, 250, 0.1)" },
};

export function ProjectCard({ id, title, status, patent_type, updated_at }: ProjectCardProps) {
  const statusCfg = STATUS_CONFIG[status] || STATUS_CONFIG.draft;

  return (
    <Link
      href={status === "draft" ? `/projects/new?id=${id}` : `/projects/${id}`}
      className="group block rounded border p-5 transition-all duration-200 hover:-translate-y-0.5"
      style={{
        borderColor: "var(--border)",
        background: "var(--surface)",
      }}
      onMouseEnter={(e) => { e.currentTarget.style.borderColor = "var(--accent-dim)"; }}
      onMouseLeave={(e) => { e.currentTarget.style.borderColor = "var(--border)"; }}
    >
      <div className="flex items-start justify-between">
        <div className="space-y-1.5 min-w-0 flex-1">
          <h3 className="text-sm font-medium text-zinc-100 truncate group-hover:text-white transition-colors">
            {title}
          </h3>
          <div className="flex items-center gap-3">
            <span className="text-xs text-zinc-600 font-mono">
              {patent_type === "provisional" ? "PROVISIONAL" : "COMPLETE"}
            </span>
            <span className="text-zinc-700">&middot;</span>
            <span className="text-xs text-zinc-600">
              {new Date(updated_at).toLocaleDateString("en-IN", { day: "numeric", month: "short", year: "numeric" })}
            </span>
          </div>
        </div>
        <span
          className="shrink-0 ml-4 inline-flex items-center rounded px-2 py-0.5 text-xs font-medium"
          style={{ color: statusCfg.color, background: statusCfg.bg }}
        >
          {statusCfg.label}
        </span>
      </div>

      {/* Subtle bottom accent line on hover */}
      <div className="mt-4 h-px w-0 group-hover:w-full transition-all duration-300" style={{ background: "var(--accent-dim)" }} />
    </Link>
  );
}
