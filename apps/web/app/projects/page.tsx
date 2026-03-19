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
      <div className="flex items-center justify-center py-20">
        <p className="text-sm text-zinc-400">Loading projects...</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-lg font-semibold text-zinc-100">Your Projects</h1>
        <button
          onClick={handleCreate}
          className="rounded-md bg-zinc-100 px-3 py-1.5 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
        >
          New Patent Draft
        </button>
      </div>

      {projects.length === 0 ? (
        <div className="rounded-lg border border-dashed border-zinc-700 p-12 text-center">
          <h2 className="text-sm font-medium text-zinc-300">No projects yet</h2>
          <p className="mt-1 text-sm text-zinc-500">
            Create your first patent draft to get started.
          </p>
          <button
            onClick={handleCreate}
            className="mt-4 rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-zinc-200"
          >
            Create your first patent draft
          </button>
        </div>
      ) : (
        <div className="space-y-3">
          {projects.map((p) => (
            <ProjectCard key={p.id} {...p} />
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
