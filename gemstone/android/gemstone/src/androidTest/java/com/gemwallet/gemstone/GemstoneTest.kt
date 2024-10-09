package com.gemwallet.gemstone

import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.*
import uniffi.Gemstone.libVersion
import uniffi.Gemstone.*

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
