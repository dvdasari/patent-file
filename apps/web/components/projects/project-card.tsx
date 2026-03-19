"use client";

import Link from "next/link";

interface ProjectCardProps {
  id: string;
  title: string;
  status: string;
  patent_type: string;
  updated_at: string;
}

const STATUS_LABELS: Record<string, string> = {
  draft: "Draft",
  interview_complete: "Ready to Generate",
  generating: "Generating...",
  review: "In Review",
  exported: "Exported",
};

export function ProjectCard({ id, title, status, patent_type, updated_at }: ProjectCardProps) {
  return (
    <Link
      href={`/projects/${id}`}
      className="block rounded-lg border border-zinc-800 bg-zinc-900 p-4 hover:border-zinc-600 transition-colors"
    >
      <div className="flex items-start justify-between">
        <div className="space-y-1">
          <h3 className="text-sm font-medium text-zinc-100">{title}</h3>
          <p className="text-xs text-zinc-500">
            {patent_type === "provisional" ? "Provisional" : "Complete"} Specification
          </p>
        </div>
        <span className="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium bg-zinc-800 text-zinc-300">
          {STATUS_LABELS[status] || status}
        </span>
      </div>
      <p className="mt-3 text-xs text-zinc-500">
        Updated {new Date(updated_at).toLocaleDateString()}
      </p>
    </Link>
  );
}
