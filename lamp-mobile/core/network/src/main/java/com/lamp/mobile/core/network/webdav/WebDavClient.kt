package com.lamp.mobile.core.network.webdav

import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.auth.*
import io.ktor.client.plugins.auth.providers.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import com.lamp.mobile.core.network.caldav.WebDavXmlParser
import java.net.URL

data class RemoteFile(
    val href: String,
    val filename: String,
    val etag: String?,
)

class WebDavClient(
    private val baseUrl: String,
    private val username: String,
    private val password: String,
) {
    private val client = HttpClient(OkHttp) {
        install(Auth) {
            basic {
                credentials { BasicAuthCredentials(username, password) }
                sendWithoutRequest { true }
            }
        }
        engine {
            config {
                connectTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
                readTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
            }
        }
    }

    private fun resolveUrl(path: String): String {
        if (path.startsWith("http://") || path.startsWith("https://")) return path
        val base = URL(baseUrl)
        return "${base.protocol}://${base.authority}$path"
    }

    suspend fun ensureCollection(): Result<Unit> = runCatching {
        val response = client.request(baseUrl) {
            method = HttpMethod("MKCOL")
        }
        // 405 = already exists, which is fine
        if (response.status.value !in listOf(201, 405, 200, 301)) {
            throw Exception("MKCOL failed: ${response.status}")
        }
    }

    suspend fun listFiles(): Result<List<RemoteFile>> = runCatching {
        val body = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:">
                <d:prop><d:getetag/></d:prop>
            </d:propfind>""".trimIndent()

        val response = client.request(baseUrl) {
            method = HttpMethod("PROPFIND")
            header("Depth", "1")
            contentType(ContentType.Application.Xml)
            setBody(body)
        }

        val xml = response.bodyAsText()
        WebDavXmlParser.parseMultistatus(xml)
            .filter { it.href != baseUrl.trimEnd('/') + "/" && it.href.endsWith(".org") }
            .map { resp ->
                RemoteFile(
                    href = resp.href,
                    filename = resp.href.trimEnd('/').substringAfterLast('/'),
                    etag = resp.etag,
                )
            }
    }

    suspend fun getFile(filename: String): Result<Pair<String, String?>> = runCatching {
        val url = "${baseUrl.trimEnd('/')}/$filename"
        val response = client.get(url)
        val content = response.bodyAsText()
        val etag = response.headers[HttpHeaders.ETag]?.trim('"')
        content to etag
    }

    suspend fun putFile(filename: String, content: String): Result<String?> = runCatching {
        val url = "${baseUrl.trimEnd('/')}/$filename"
        val response = client.put(url) {
            contentType(ContentType("text", "org", parameters = listOf(
                io.ktor.http.HeaderValueParam("charset", "utf-8")
            )))
            setBody(content)
        }
        response.headers[HttpHeaders.ETag]?.trim('"')
    }

    suspend fun deleteFile(filename: String): Result<Unit> = runCatching {
        val url = "${baseUrl.trimEnd('/')}/$filename"
        val response = client.delete(url)
        if (response.status.value !in listOf(200, 204, 404)) {
            throw Exception("DELETE failed: ${response.status}")
        }
    }

    fun close() = client.close()
}
