package com.example.gemtest

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.example.gemtest.ui.theme.GemTestTheme
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.*

val Warp = AlienProviderWarp(NativeProvider())

class MainActivity : ComponentActivity() {

    init {
        System.loadLibrary("gemstone")
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            GemTestTheme {
                // A surface container using the 'background' color from the theme
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    ContentView("Gemstone lib version: " + libVersion())
                }
            }
        }
    }
}

@Composable
fun ContentView(text: String, modifier: Modifier = Modifier) {
    Column(modifier = Modifier.fillMaxSize()) {
        Text(
            text = text,
            modifier = modifier
        )
        Button(
            onClick = { fetchData() },
            modifier = Modifier.size(width = 120.dp, height = 80.dp)
        ) {
            Text(text = "Fetch Data")
        }
    }
}

fun fetchData() {
    println("Kotlin <> Rust")
    runBlocking {
        val target = AlienTarget(
            url = "https://httpbin.org/get?foo=bar",
            method = "GET",
            headers = hashMapOf(
                "X-Header" to "X-Value"
            ),
            body = null
        )
        val data = Warp.teleport(listOf(target))
        println(String(data[0]))
    }
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    GemTestTheme {
        ContentView("Android")
    }
}