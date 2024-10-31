package com.gemwallet.gemstone

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.*
import uniffi.gemstone.*

@RunWith(AndroidJUnit4::class)
class GemstoneTest {

    init {
        System.loadLibrary("gemstone")
    }

    @Test
    fun testLibVersion() {
        assertTrue(libVersion().isNotEmpty())
    }
}