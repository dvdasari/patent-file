const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:5012";

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
  });

  if (res.status === 401) {
    window.location.href = "/login";
    throw new Error("Unauthorized");
  }
  if (res.status === 403) {
    throw new Error("Subscription required");
  }
  if (res.status === 429) {
    throw new Error("Rate limit exceeded. Please try again later.");
  }
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body.error || `Request failed: ${res.status}`);
  }

  return res.json();
}

export const api = {
  // Auth
  login: (data: { email: string; password: string }) =>
    request("/api/auth/login", { method: "POST", body: JSON.stringify(data) }),
  logout: () => request("/api/auth/logout", { method: "POST" }),
  refresh: () => request("/api/auth/refresh", { method: "POST" }),

  // User
  getMe: () => request<{
    id: string;
    email: string;
    full_name: string;
    has_active_subscription: boolean;
  }>("/api/me"),

  // Subscriptions
  createSubscription: () =>
    request<{ subscription_id: string }>("/api/subscriptions/create", {
      method: "POST",
    }),

  // Projects
  listProjects: () => request<{ projects: unknown[] }>("/api/projects"),
  createProject: (data: {
    title: string;
    patent_type: string;
    jurisdiction?: string;
  }) =>
    request("/api/projects", { method: "POST", body: JSON.stringify(data) }),
  getProject: (id: string) => request(`/api/projects/${id}`),
  updateProject: (id: string, data: Record<string, string>) =>
    request(`/api/projects/${id}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),
  deleteProject: (id: string) =>
    request(`/api/projects/${id}`, { method: "DELETE" }),

  // Applicant details
  getApplicant: (projectId: string) =>
    request(`/api/projects/${projectId}/applicant`),
  upsertApplicant: (projectId: string, data: Record<string, unknown>) =>
    request(`/api/projects/${projectId}/applicant`, {
      method: "PUT",
      body: JSON.stringify(data),
    }),

  // Interview
  getInterview: (projectId: string) =>
    request(`/api/projects/${projectId}/interview`),
  saveInterview: (projectId: string, responses: unknown[]) =>
    request(`/api/projects/${projectId}/interview`, {
      method: "PUT",
      body: JSON.stringify({ responses }),
    }),

  // Sections
  updateSection: (projectId: string, sectionType: string, content: string) =>
    request(`/api/projects/${projectId}/sections/${sectionType}`, {
      method: "PUT",
      body: JSON.stringify({ content }),
    }),

  // Section versions
  listVersions: (projectId: string, sectionType: string) =>
    request(`/api/projects/${projectId}/sections/${sectionType}/versions`),
  restoreVersion: (
    projectId: string,
    sectionType: string,
    versionNumber: number
  ) =>
    request(
      `/api/projects/${projectId}/sections/${sectionType}/versions/${versionNumber}/restore`,
      { method: "POST" }
    ),

  // Export
  createExport: (projectId: string, format: string) =>
    request(`/api/projects/${projectId}/export`, {
      method: "POST",
      body: JSON.stringify({ format }),
    }),
  listExports: (projectId: string) =>
    request(`/api/projects/${projectId}/exports`),
  getDownloadUrl: (exportId: string) =>
    request<{ url: string }>(`/api/exports/${exportId}/download`),
};
