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

  // Prior Art Search
  createSearch: (data: {
    query: string;
    ipc_classification?: string;
    applicant?: string;
    date_from?: string;
    date_to?: string;
    include_npl?: boolean;
    project_id?: string;
  }) =>
    request<SearchResponse>("/api/search", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  listSearches: () =>
    request<PriorArtSearch[]>("/api/searches"),
  getSearch: (id: string) =>
    request<SearchResponse>(`/api/searches/${id}`),
  generateSearchReport: (searchId: string) =>
    request<SearchReport>(`/api/searches/${searchId}/report`, {
      method: "POST",
    }),
  downloadSearchReport: (reportId: string) =>
    request<{ url: string }>(`/api/search-reports/${reportId}/download`),

  // FER Analysis
  createFer: (data: {
    fer_text: string;
    title?: string;
    application_number?: string;
    fer_date?: string;
    project_id?: string;
  }) =>
    request<FerAnalysis>("/api/fer", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  listFer: () => request<FerAnalysis[]>("/api/fer"),
  getFer: (id: string) => request<FerAnalysisDetail>(`/api/fer/${id}`),
  updateFerResponse: (responseId: string, data: { user_edited_text: string }) =>
    request<FerResponseItem>(`/api/fer/responses/${responseId}`, {
      method: "PATCH",
      body: JSON.stringify(data),
    }),
  acceptFerResponse: (responseId: string) =>
    request<FerResponseItem>(`/api/fer/responses/${responseId}/accept`, {
      method: "POST",
    }),
};

// Search types
export interface PriorArtSearch {
  id: string;
  user_id: string;
  project_id: string | null;
  query_text: string;
  ipc_classification: string | null;
  applicant_filter: string | null;
  date_from: string | null;
  date_to: string | null;
  include_npl: boolean;
  status: string;
  result_count: number;
  created_at: string;
  updated_at: string;
}

export interface PriorArtResult {
  id: string;
  search_id: string;
  source: string;
  external_id: string | null;
  title: string;
  applicant: string | null;
  filing_date: string | null;
  publication_date: string | null;
  ipc_codes: string | null;
  abstract_text: string | null;
  url: string | null;
  similarity_score: number;
  novelty_assessment: string | null;
  relevance_rank: number;
  created_at: string;
}

export interface SearchReport {
  id: string;
  search_id: string;
  format: string;
  storage_path: string;
  file_size_bytes: number;
  created_at: string;
}

export interface SearchResponse {
  search: PriorArtSearch;
  results: PriorArtResult[];
}

// FER types
export interface FerAnalysis {
  id: string;
  user_id: string;
  project_id: string | null;
  title: string;
  fer_text: string;
  application_number: string | null;
  fer_date: string | null;
  examiner_name: string | null;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface FerObjection {
  id: string;
  analysis_id: string;
  objection_number: number;
  category: string;
  section_reference: string | null;
  summary: string;
  full_text: string;
  created_at: string;
}

export interface FerResponseItem {
  id: string;
  objection_id: string;
  legal_arguments: string;
  claim_amendments: string;
  case_law_citations: string;
  status: string;
  user_edited_text: string | null;
  created_at: string;
  updated_at: string;
}

export interface ObjectionWithResponse extends FerObjection {
  response: FerResponseItem | null;
}

export interface FerAnalysisDetail extends FerAnalysis {
  objections: ObjectionWithResponse[];
}
