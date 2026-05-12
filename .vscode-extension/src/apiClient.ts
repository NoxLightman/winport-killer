import { ProcessResponse, PortResponse, KillResult, VersionResponse, HealthResponse, ViewMode } from "./types";

export class ApiClient {
  constructor(private readonly baseUrl: string) {}

  async health(): Promise<HealthResponse> {
    return this.getJson<HealthResponse>("/health");
  }

  async version(): Promise<VersionResponse> {
    return this.getJson<VersionResponse>("/version");
  }

  async fetchView(mode: ViewMode, filter: string): Promise<PortResponse | ProcessResponse> {
    const search = filter ? `?filter=${encodeURIComponent(filter)}` : "";
    return this.getJson<PortResponse | ProcessResponse>(`/${mode}${search}`);
  }

  async kill(pid: number): Promise<KillResult> {
    const response = await fetch(`${this.baseUrl}/kill/${pid}`, { method: "POST" });
    if (!response.ok) {
      throw new Error(`Kill request failed: ${response.status}`);
    }
    return response.json() as Promise<KillResult>;
  }

  private async getJson<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`);
    if (!response.ok) {
      throw new Error(`Request failed: ${response.status} ${path}`);
    }
    return response.json() as Promise<T>;
  }
}
