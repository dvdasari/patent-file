"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { api } from "@/lib/api-client";
import { AuthGuard } from "@/components/layout/auth-guard";
import { SubscriptionGuard } from "@/components/layout/subscription-guard";
import { Navbar } from "@/components/layout/navbar";
import { ProjectCard } from "@/components/projects/project-card";

interface Project {
  id: string;
  title: string;
  status: string;
  patent_type: string;
  updated_at: string;
}

function ProjectsContent() {
  const router = useRouter();
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.listProjects().then((data) => {
      setProjects((data as unknown as Project[]) || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  async function handleCreate() {
    try {
      const project = await api.createProject({
        title: "Untitled Patent",
        patent_type: "complete",
      }) as unknown as Project;
      router.push(`/projects/new?id=${project.id}`);
    } catch (err) {
      console.error("Failed to create project:", err);
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-32">
        <div className="flex items-center gap-3">
          <div className="w-3 h-3 border border-zinc-600 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-zinc-500">Loading projects...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-6 py-10 animate-fade-in">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-lg font-medium tracking-tight text-zinc-100">Projects</h1>
          <p className="text-xs text-zinc-500 mt-1">{projects.length} patent draft{projects.length !== 1 ? "s" : ""}</p>
        </div>
        <button
          onClick={handleCreate}
          className="rounded px-4 py-2 text-xs font-medium transition-all duration-200"
          style={{ background: "var(--accent)", color: "var(--background)" }}
          onMouseEnter={(e) => { e.currentTarget.style.opacity = "0.9"; }}
          onMouseLeave={(e) => { e.currentTarget.style.opacity = "1"; }}
        >
          + New Draft
        </button>
      </div>

      {projects.length === 0 ? (
        <div className="rounded border border-dashed p-16 text-center" style={{ borderColor: "var(--border)" }}>
          <div className="mx-auto w-12 h-12 rounded-full flex items-center justify-center mb-4" style={{ background: "rgba(200, 169, 110, 0.08)" }}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" style={{ color: "var(--accent)" }}>
              <path d="M9 12h6m-3-3v6m-7 4h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
          </div>
          <h2 className="text-sm font-medium text-zinc-300 mb-1">No drafts yet</h2>
          <p className="text-xs text-zinc-600 mb-6">
            Create your first patent draft to get started
          </p>
          <button
            onClick={handleCreate}
            className="rounded px-5 py-2.5 text-xs font-medium transition-all"
            style={{ background: "var(--accent)", color: "var(--background)" }}
          >
            Create your first draft
          </button>
        </div>
      ) : (
        <div className="space-y-2">
          {projects.map((p, i) => (
            <div key={p.id} className="animate-fade-in" style={{ animationDelay: `${i * 50}ms` }}>
              <ProjectCard {...p} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default function ProjectsPage() {
  return (
    <AuthGuard>
      <SubscriptionGuard>
        <Navbar />
        <ProjectsContent />
      </SubscriptionGuard>
    </AuthGuard>
  );
}
