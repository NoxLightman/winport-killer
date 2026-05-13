package dev.winportkill.jetbrains.api

import dev.winportkill.jetbrains.model.HealthResponse
import dev.winportkill.jetbrains.model.KillResult
import dev.winportkill.jetbrains.model.PortResponse
import dev.winportkill.jetbrains.model.ProcessResponse
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json
import java.net.URI
import java.net.URLEncoder
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.charset.StandardCharsets
import java.time.Duration

class ApiClient(
    private val baseUrl: String,
) {
    private val httpClient = HttpClient.newBuilder()
        .connectTimeout(Duration.ofSeconds(2))
        .build()

    private val json = Json {
        ignoreUnknownKeys = true
    }

    fun health(): HealthResponse = getJson("/health", HealthResponse.serializer())

    fun fetchPorts(filter: String): PortResponse = getJson(
        pathWithFilter("/ports", filter),
        PortResponse.serializer()
    )

    fun fetchProcesses(filter: String): ProcessResponse = getJson(
        pathWithFilter("/processes", filter),
        ProcessResponse.serializer()
    )

    fun kill(pid: Int): KillResult {
        val request = HttpRequest.newBuilder(URI.create("$baseUrl/kill/$pid"))
            .timeout(Duration.ofSeconds(5))
            .POST(HttpRequest.BodyPublishers.noBody())
            .build()

        val response = httpClient.send(request, HttpResponse.BodyHandlers.ofString())
        check(response.statusCode() in 200..299) {
            "Kill request failed: ${response.statusCode()}"
        }
        return json.decodeFromString(KillResult.serializer(), response.body())
    }

    private fun <T> getJson(path: String, serializer: KSerializer<T>): T {
        val request = HttpRequest.newBuilder(URI.create("$baseUrl$path"))
            .timeout(Duration.ofSeconds(5))
            .GET()
            .build()

        val response = httpClient.send(request, HttpResponse.BodyHandlers.ofString())
        check(response.statusCode() in 200..299) {
            "Request failed: ${response.statusCode()} $path"
        }
        return json.decodeFromString(serializer, response.body())
    }

    private fun pathWithFilter(path: String, filter: String): String {
        if (filter.isBlank()) {
            return path
        }
        val encoded = URLEncoder.encode(filter, StandardCharsets.UTF_8)
        return "$path?filter=$encoded"
    }
}
