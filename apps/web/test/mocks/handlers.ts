import { http, HttpResponse } from "msw";

const API_URL = "http://localhost:5012";

export const handlers = [
  http.get(`${API_URL}/api/health`, () => {
    return HttpResponse.json({ status: "ok" });
  }),
];
