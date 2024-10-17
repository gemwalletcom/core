package com.example.gemtest

import io.ktor.client.*
import io.ktor.client.call.body
import io.ktor.client.engine.cio.*
import io.ktor.client.request.*
import io.ktor.http.*
import uniffi.Gemstone.*

class NativeProvider: AlienProvider {
    val client = HttpClient(CIO) {
        expectSuccess = true
    }

    fun close() {
        client.close()
    }

    override suspend fun request(target: AlienTarget): ByteArray {
        val parsedUrl = parseUrl(target.url) ?: return ByteArray(0)
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
        return response.body()
    }
}
