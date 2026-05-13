package dev.winportkill.jetbrains.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class HealthResponse(
    val status: String,
)

@Serializable
data class KillResult(
    val success: Boolean,
    val message: String,
)

@Serializable
data class PortResponse(
    val entries: List<PortEntry>,
    val stats: PortStats,
)

@Serializable
data class ProcessResponse(
    val entries: List<ProcessEntry>,
    val stats: ProcessStats,
)

@Serializable
data class PortEntry(
    val proto: String,
    @SerialName("local_addr")
    val localAddr: String,
    val port: String,
    val pid: Int,
    val name: String,
    val memory: Long,
)

@Serializable
data class ProcessEntry(
    val pid: Int,
    val name: String,
    val memory: Long,
    @SerialName("tcp_ports")
    val tcpPorts: Int,
    @SerialName("udp_ports")
    val udpPorts: Int,
    val ports: List<PortBinding>,
)

@Serializable
data class PortBinding(
    val proto: String,
    @SerialName("local_addr")
    val localAddr: String,
    val port: String,
)

@Serializable
data class PortStats(
    @SerialName("total_rows")
    val totalRows: Int,
    @SerialName("total_procs")
    val totalProcs: Int,
    @SerialName("tcp_count")
    val tcpCount: Int,
    @SerialName("udp_count")
    val udpCount: Int,
    @SerialName("total_mem_bytes")
    val totalMemBytes: Long,
)

@Serializable
data class ProcessStats(
    @SerialName("total_procs")
    val totalProcs: Int,
    @SerialName("procs_with_ports")
    val procsWithPorts: Int,
    @SerialName("total_port_bindings")
    val totalPortBindings: Int,
    @SerialName("tcp_count")
    val tcpCount: Int,
    @SerialName("udp_count")
    val udpCount: Int,
    @SerialName("total_mem_bytes")
    val totalMemBytes: Long,
)

enum class ViewMode {
    PORTS,
    PROCESSES,
}
