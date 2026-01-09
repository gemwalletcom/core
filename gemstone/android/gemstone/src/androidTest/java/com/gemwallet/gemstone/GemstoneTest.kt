package com.gemwallet.gemstone

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.*
import java.net.SocketTimeoutException

/**
 * Mock provider for testing exception handling in AlienProvider implementations.
 */
class MockProvider(
    private val onRequest: suspend (AlienTarget) -> AlienResponse = {
        AlienResponse(status = 200u, data = ByteArray(0))
    }
) : AlienProvider {
    override suspend fun request(target: AlienTarget): AlienResponse = onRequest(target)
    override fun getEndpoint(chain: Chain): String = "https://mock.endpoint"
}

/**
 * Mock preferences for GemGateway.
 */
class MockPreferences : GemPreferences {
    override fun get(key: String): String? = null
    override fun set(key: String, value: String) {}
    override fun remove(key: String) {}
}

// Simulating a custom app exception (e.g., BlockchainError.DustError)
sealed class CustomAppException : Exception() {
    class DustError(override val message: String) : CustomAppException()
}

@RunWith(AndroidJUnit4::class)
class GemstoneTest {

    init {
        System.loadLibrary("gemstone")
    }

    private fun createGateway(provider: AlienProvider): GemGateway {
        return GemGateway(provider, MockPreferences(), MockPreferences(), "https://api.example.com")
    }

    @Test
    fun testLibVersion() {
        assertTrue(libVersion().isNotEmpty())
    }

    /**
     * Test 1: UniFFI-defined exception is caught correctly.
     */
    @Test
    fun testProviderThrowsAlienException() = runBlocking {
        val errorMessage = "Request failed"
        val provider = MockProvider { throw AlienException.RequestException(errorMessage) }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected GatewayException")
        } catch (e: GatewayException.NetworkException) {
            assertTrue(e.msg.contains(errorMessage))
        }
    }

    /**
     * Test 2: Standard Java exception is wrapped and caught.
     */
    @Test
    fun testProviderThrowsStandardException() = runBlocking {
        val errorMessage = "Network timeout"
        val provider = MockProvider { throw SocketTimeoutException(errorMessage) }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected exception")
        } catch (e: GatewayException.NetworkException) {
            assertTrue(e.msg.contains(errorMessage))
        } catch (e: InternalException) {
            assertTrue(e.message?.contains(errorMessage) == true)
        }
    }

    /**
     * Test 3: Custom exceptions must be wrapped to avoid native crash.
     *
     * WARNING: Throwing custom exceptions directly causes native crash.
     * This is the root cause of production crashes like:
     * UnexpectedUniFFICallbackError(reason: "...BlockchainError$DustError")
     */
    @Test
    fun testProviderWrapsCustomException() = runBlocking {
        val errorMessage = "Amount too small"
        val provider = MockProvider {
            try {
                throw CustomAppException.DustError(errorMessage)
            } catch (e: CustomAppException) {
                throw AlienException.RequestException("${e::class.simpleName}: ${e.message}")
            }
        }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected GatewayException")
        } catch (e: GatewayException.NetworkException) {
            assertTrue(e.msg.contains(errorMessage))
        }
    }
}
