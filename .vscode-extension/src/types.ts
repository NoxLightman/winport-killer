export type ViewMode = "ports" | "processes";

export interface PortEntry {
  proto: string;
  local_addr: string;
  port: string;
  pid: number;
  name: string;
  memory: number;
}

export interface PortStats {
  total_rows: number;
  total_procs: number;
  tcp_count: number;
  udp_count: number;
  total_mem_bytes: number;
}

export interface PortResponse {
  entries: PortEntry[];
  stats: PortStats;
}

export interface PortBinding {
  proto: string;
  local_addr: string;
  port: string;
}

export interface ProcessEntry {
  pid: number;
  name: string;
  memory: number;
  tcp_ports: number;
  udp_ports: number;
  ports: PortBinding[];
}

export interface ProcessStats {
  total_procs: number;
  procs_with_ports: number;
  total_port_bindings: number;
  tcp_count: number;
  udp_count: number;
  total_mem_bytes: number;
}

export interface ProcessResponse {
  entries: ProcessEntry[];
  stats: ProcessStats;
}

export interface KillResult {
  success: boolean;
  message: string;
}

export interface VersionResponse {
  name: string;
  version: string;
}

export interface HealthResponse {
  status: string;
}
