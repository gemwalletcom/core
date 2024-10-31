package com.example.gemtest

import io.ktor.client.*
import io.ktor.client.call.body
import io.ktor.client.engine.cio.*
import io.ktor.client.request.*
import io.ktor.http.*
import uniffi.gemstone.*

class NativeProvider: AlienProvider {
    val client = HttpClient(CIO) {
        expectSuccess = true
    }

    fun close() {
        client.close()
    }

    override fun getEndpoint(chain: Chain): String {
        return "http://localhost:8080"
    }

    override suspend fun request(targets: List<AlienTarget>): List<ByteArray> {
        val results = mutableListOf<ByteArray>()
        for (target in targets) {
            val parsedUrl = parseUrl(target.url) ?: throw Exception("Invalid url: ${target.url}")
            val response = client.request {
                method = HttpMethod(target.method)
                url {
                    protocolOrNull = parsedUrl.protocol
                    host = parsedUrl.host
                    path(parsedUrl.encodedPath)
                    parameters.appendAll(parsedUrl.parameters)
                }
                headers {
                    for ((key, value) in target.headers ?: HashMap<String, String>()) {
                        append(key, value)
                    }
                }
                setBody(
                    target.body ?: ""
                )
            }
            results.add(response.body())
        }
        return results
    }
}
