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

sealed class CustomAppException : Exception() {
    class DustError(override val message: String) : CustomAppException()
}

/**
 * Mock fee estimator for testing GatewayException.PlatformException.
 */
class MockFeeEstimator(
    private val onGetFee: suspend (Chain, GemTransactionLoadInput) -> GemTransactionLoadFee? = { _, _ -> null }
) : GemGatewayEstimateFee {
    override suspend fun getFee(chain: Chain, input: GemTransactionLoadInput): GemTransactionLoadFee? = onGetFee(chain, input)
    override suspend fun getFeeData(chain: Chain, input: GemTransactionLoadInput): String? = null
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
     * Test 1: UniFFI-defined exception (AlienException) is caught as GatewayException.
     */
    @Test
    fun testProviderThrowsAlienException() = runBlocking {
        val errorMessage = "Request failed"
        val provider = MockProvider { throw AlienException.RequestException(errorMessage) }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected GatewayException.NetworkException to be thrown")
        } catch (e: GatewayException.NetworkException) {
            assertTrue(e.msg.contains(errorMessage))
        }
    }

    /**
     * Test 2: Standard Java exception becomes InternalException.
     *
     * Note: Unlike AlienException which is properly mapped to GatewayException,
     * standard Java exceptions result in InternalException with UnexpectedUniFFICallbackError.
     */
    @Test
    fun testProviderThrowsStandardException() = runBlocking {
        val errorMessage = "Network timeout"
        val provider = MockProvider { throw SocketTimeoutException(errorMessage) }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected InternalException to be thrown")
        } catch (e: InternalException) {
            assertTrue(e.message?.contains(errorMessage) ?: false)
        }
    }

    /**
     * Test 3: Custom exceptions must be wrapped to avoid native crash.
     *
     * Throwing custom exceptions directly causes native crash
     * like UnexpectedUniFFICallbackError(reason: "...BlockchainError$DustError")
     * Use GatewayException.PlatformException to wrap custom exceptions, allowing
     * clients to distinguish them from network errors.
     */
    @Test
    fun testFeeEstimatorThrowsPlatformException() = runBlocking {
        val errorMessage = "Amount too small"
        val feeEstimator = MockFeeEstimator { _, _ ->
            // Custom exceptions must be wrapped to avoid native crash
            try {
                throw CustomAppException.DustError(errorMessage)
            } catch (e: CustomAppException) {
                throw GatewayException.PlatformException("${e::class.simpleName}: ${e.message}")
            }
        }
        val gateway = createGateway(MockProvider())
        val asset = GemAsset(
            id = "ethereum",
            chain = "ethereum",
            tokenId = null,
            name = "Ethereum",
            symbol = "ETH",
            decimals = 18,
            assetType = GemAssetType.NATIVE
        )
        val input = GemTransactionLoadInput(
            inputType = GemTransactionInputType.Transfer(asset),
            senderAddress = "0x1234",
            destinationAddress = "0x5678",
            value = "1000000000000000000",
            gasPrice = GemGasPriceType.Regular("20000000000"),
            memo = null,
            isMaxValue = false,
            metadata = GemTransactionLoadMetadata.None
        )

        try {
            gateway.getFee("ethereum", input, feeEstimator)
            fail("Expected GatewayException.PlatformException to be thrown")
        } catch (e: GatewayException.PlatformException) {
            assertTrue(e.msg.contains(errorMessage))
        }
    }
}
