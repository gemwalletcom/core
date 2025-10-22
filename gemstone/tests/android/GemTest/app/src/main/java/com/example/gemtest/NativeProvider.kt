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

    override suspend fun request(target: AlienTarget): AlienResponse {
        val parsedUrl = try {
            Url(target.url)
        } catch (e: Throwable) {
            throw AlienError.RequestError("invalid url: ${target.url}")
        }

        val response = client.request {
            method = HttpMethod(target.method)
            url.takeFrom(parsedUrl)
            headers {
                target.headers?.forEach { (key, value) -> append(key, value) }
            }
            target.body?.let { setBody(it) }
        }

        val bytes: ByteArray = response.body()
        val status = response.status.value

        return AlienResponse(status.toUShort(), bytes)
    }
}
