package com.lamp.mobile.core.network.caldav

import com.lamp.mobile.core.model.CalendarInfo
import android.util.Log
import com.lamp.mobile.core.network.MethodPreservingRedirectInterceptor
import com.lamp.mobile.core.network.upgradeToHttps
import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.auth.*
import io.ktor.client.plugins.auth.providers.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import java.net.URL

data class RemoteVtodo(
    val href: String,
    val etag: String,
    val uid: String?,
    val icalBody: String,
)

sealed class SyncChange {
    data class Changed(val vtodo: RemoteVtodo) : SyncChange()
    data class Deleted(val href: String) : SyncChange()
}

enum class PutCondition {
    CREATE_ONLY,    // If-None-Match: *
    UNCONDITIONAL,  // No conditional header
}

class CalDavClient(
    baseUrl: String,
    private val username: String,
    private val password: String,
) {
    private val baseUrl = baseUrl.upgradeToHttps()
    private val client = HttpClient(OkHttp) {
        install(Auth) {
            basic {
                credentials {
                    BasicAuthCredentials(
                        username = this@CalDavClient.username,
                        password = this@CalDavClient.password,
                    )
                }
                sendWithoutRequest { true }
            }
        }
        engine {
            config {
                followRedirects(false)
                connectTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
                readTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
            }
            addInterceptor(MethodPreservingRedirectInterceptor())
        }
    }

    private fun resolveUrl(path: String): String {
        if (path.startsWith("https://")) return path
        if (path.startsWith("http://")) return path.upgradeToHttps()
        val base = URL(baseUrl)
        return "https://${base.authority}$path"
    }

    // --- Discovery ---

    suspend fun discoverCalendars(): Result<List<CalendarInfo>> = runCatching {
        val principal = findPrincipal()
        val homeSet = findCalendarHomeSet(principal)
        listCalendars(homeSet)
    }

    private suspend fun findPrincipal(): String {
        val errors = mutableListOf<String>()

        // Try .well-known first (RFC 6764) — redirects followed by interceptor
        try {
            val url = resolveUrl("/.well-known/caldav")
            Log.d("CalDav", "PROPFIND principal at: $url")
            val body = propfind(url, 0, PROPFIND_PRINCIPAL)
            Log.d("CalDav", "PROPFIND principal response: ${body.take(500)}")
            val principal = WebDavXmlParser.extractHref(body, "current-user-principal")
            Log.d("CalDav", "Extracted principal: $principal")
            if (principal != null) return principal
        } catch (e: Exception) {
            Log.w("CalDav", ".well-known failed: ${e.message}")
            errors.add(".well-known/caldav: ${e.message}")
        }

        // Fallback to PROPFIND on base URL
        try {
            Log.d("CalDav", "PROPFIND principal at base: $baseUrl")
            val body = propfind(baseUrl, 0, PROPFIND_PRINCIPAL)
            Log.d("CalDav", "PROPFIND base response: ${body.take(500)}")
            val principal = WebDavXmlParser.extractHref(body, "current-user-principal")
            Log.d("CalDav", "Extracted principal from base: $principal")
            if (principal != null) return principal
        } catch (e: Exception) {
            Log.w("CalDav", "base PROPFIND failed: ${e.message}")
            errors.add("base URL: ${e.message}")
        }

        // Fastmail-style fallback
        val fallback = "/dav/principals/user/$username/"
        try {
            Log.d("CalDav", "Trying Fastmail-style principal: $fallback")
            val body = propfind(resolveUrl(fallback), 0, PROPFIND_PRINCIPAL)
            val principal = WebDavXmlParser.extractHref(body, "current-user-principal")
            if (principal != null) return principal
            // Server responded with valid XML but no principal — try using the path directly
            return fallback
        } catch (e: Exception) {
            Log.w("CalDav", "Fastmail fallback failed: ${e.message}")
            errors.add("fallback: ${e.message}")
        }

        throw Exception("Could not find CalDAV principal. ${errors.joinToString("; ")}")
    }

    private suspend fun findCalendarHomeSet(principal: String): String {
        val url = resolveUrl(principal)
        Log.d("CalDav", "PROPFIND calendar-home-set at: $url")
        val body = propfind(url, 0, PROPFIND_HOME_SET)
        Log.d("CalDav", "calendar-home-set response: ${body.take(500)}")
        val homeSet = WebDavXmlParser.extractHref(body, "calendar-home-set")
        Log.d("CalDav", "Extracted calendar-home-set: $homeSet")
        return homeSet ?: throw Exception("No calendar-home-set found at $url")
    }

    private suspend fun listCalendars(homeSet: String): List<CalendarInfo> {
        val url = resolveUrl(homeSet)
        Log.d("CalDav", "PROPFIND calendars at: $url")
        val body = propfind(url, 1, PROPFIND_CALENDARS)
        Log.d("CalDav", "Calendar listing response: ${body.take(2000)}")
        val result = WebDavXmlParser.parseCalendarListing(body)
        Log.d("CalDav", "Parsed ${result.size} calendars: ${result.map { "${it.displayName} vtodo=${it.supportsVtodo} vevent=${it.supportsVevent}" }}")
        return result
    }

    // --- CRUD ---

    suspend fun listVtodos(calendarHref: String): Result<List<RemoteVtodo>> = runCatching {
        val body = report(resolveUrl(calendarHref), REPORT_VTODOS)
        WebDavXmlParser.parseMultistatus(body).mapNotNull { resp ->
            if (resp.calendarData != null && resp.etag != null) {
                RemoteVtodo(resp.href, resp.etag, extractUid(resp.calendarData), resp.calendarData)
            } else null
        }
    }

    suspend fun listVevents(calendarHref: String): Result<List<RemoteVtodo>> = runCatching {
        val body = report(resolveUrl(calendarHref), REPORT_VEVENTS)
        WebDavXmlParser.parseMultistatus(body).mapNotNull { resp ->
            if (resp.calendarData != null && resp.etag != null) {
                RemoteVtodo(resp.href, resp.etag, extractUid(resp.calendarData), resp.calendarData)
            } else null
        }
    }

    suspend fun putVtodo(href: String, condition: PutCondition, etag: String?, ical: String): Result<String> = runCatching {
        val response = client.put(resolveUrl(href)) {
            contentType(ContentType("text", "calendar", parameters = listOf(HeaderValueParam("charset", "utf-8"))))
            setBody(ical)
            when (condition) {
                PutCondition.CREATE_ONLY -> header(HttpHeaders.IfNoneMatch, "*")
                PutCondition.UNCONDITIONAL -> {}
            }
            if (etag != null) {
                header(HttpHeaders.IfMatch, "\"$etag\"")
            }
        }
        if (!response.status.isSuccess()) {
            throw Exception("PUT failed: ${response.status}")
        }
        response.headers[HttpHeaders.ETag]?.trim('"') ?: ""
    }

    suspend fun deleteVtodo(href: String, etag: String): Result<Unit> = runCatching {
        val response = client.delete(resolveUrl(href)) {
            header(HttpHeaders.IfMatch, "\"$etag\"")
        }
        if (response.status != HttpStatusCode.NoContent && response.status != HttpStatusCode.OK &&
            response.status != HttpStatusCode.NotFound) {
            throw Exception("DELETE failed: ${response.status}")
        }
    }

    // --- Sync Collection (RFC 6578) ---

    suspend fun syncCollection(calendarHref: String, syncToken: String?): Result<Pair<List<SyncChange>, String?>> = runCatching {
        val tokenXml = if (syncToken != null) "<d:sync-token>$syncToken</d:sync-token>"
        else "<d:sync-token/>"

        val body = """<?xml version="1.0" encoding="utf-8"?>
            <d:sync-collection xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
                $tokenXml
                <d:sync-level>1</d:sync-level>
                <d:prop>
                    <d:getetag/>
                    <c:calendar-data/>
                </d:prop>
            </d:sync-collection>""".trimIndent()

        val response = client.request(resolveUrl(calendarHref)) {
            method = HttpMethod("REPORT")
            header("Depth", "1")
            contentType(ContentType.Application.Xml)
            setBody(body)
        }

        if (response.status.value in listOf(403, 409, 412)) {
            val respBody = response.bodyAsText()
            if (respBody.contains("valid-sync-token")) {
                throw SyncTokenExpiredException()
            }
            throw Exception("Sync collection failed: ${response.status}")
        }

        val xml = response.bodyAsText()
        requireXml(xml, response.status)
        val newToken = WebDavXmlParser.extractSyncToken(xml)

        val changes = WebDavXmlParser.parseMultistatus(xml).map { resp ->
            if (resp.status?.contains("404") == true) {
                SyncChange.Deleted(resp.href)
            } else if (resp.calendarData != null && resp.etag != null) {
                SyncChange.Changed(RemoteVtodo(resp.href, resp.etag, extractUid(resp.calendarData), resp.calendarData))
            } else {
                SyncChange.Deleted(resp.href)
            }
        }

        changes to newToken
    }

    /** Fetch the current sync-token for a collection via PROPFIND. */
    suspend fun getSyncToken(calendarHref: String): String? {
        val body = propfind(resolveUrl(calendarHref), 0, PROPFIND_SYNC_TOKEN)
        return WebDavXmlParser.extractSyncToken(body)
    }

    // --- Helpers ---

    private suspend fun propfind(url: String, depth: Int, body: String): String {
        val response = client.request(url) {
            method = HttpMethod("PROPFIND")
            header("Depth", depth.toString())
            contentType(ContentType.Application.Xml)
            setBody(body)
        }
        val text = response.bodyAsText()
        if (!response.status.isSuccess() && response.status != HttpStatusCode.MultiStatus) {
            throw Exception("PROPFIND $url failed: ${response.status} — ${text.take(200)}")
        }
        requireXml(text, response.status)
        return text
    }

    private suspend fun report(url: String, body: String): String {
        val response = client.request(url) {
            method = HttpMethod("REPORT")
            header("Depth", "1")
            contentType(ContentType.Application.Xml)
            setBody(body)
        }
        val text = response.bodyAsText()
        if (!response.status.isSuccess() && response.status != HttpStatusCode.MultiStatus) {
            throw Exception("REPORT $url failed: ${response.status} — ${text.take(200)}")
        }
        requireXml(text, response.status)
        return text
    }

    private fun extractUid(ical: String): String? =
        ical.lineSequence().find { it.startsWith("UID:") }?.substringAfter("UID:")

    fun close() = client.close()

    companion object {
        /** Throw if response body looks like HTML rather than XML. */
        internal fun requireXml(body: String, status: HttpStatusCode) {
            val trimmed = body.trimStart()
            if (trimmed.startsWith("<html", ignoreCase = true) ||
                trimmed.startsWith("<!doctype", ignoreCase = true) ||
                (!trimmed.startsWith("<?xml") && !trimmed.startsWith("<") )) {
                throw Exception("Server returned non-XML response ($status): ${body.take(200)}")
            }
            // Check for HTML tags that should never appear in WebDAV XML
            if ("<body" in body.lowercase() || "<hr" in body.lowercase()) {
                throw Exception("Server returned HTML instead of XML ($status): ${body.take(200)}")
            }
        }

        private val PROPFIND_PRINCIPAL = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:">
                <d:prop><d:current-user-principal/></d:prop>
            </d:propfind>""".trimIndent()

        private val PROPFIND_HOME_SET = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
                <d:prop><c:calendar-home-set/></d:prop>
            </d:propfind>""".trimIndent()

        private val PROPFIND_CALENDARS = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
                <d:prop>
                    <d:displayname/>
                    <d:resourcetype/>
                    <c:supported-calendar-component-set/>
                </d:prop>
            </d:propfind>""".trimIndent()

        private val PROPFIND_SYNC_TOKEN = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:">
                <d:prop><d:sync-token/></d:prop>
            </d:propfind>""".trimIndent()

        private val REPORT_VTODOS = """<?xml version="1.0" encoding="utf-8"?>
            <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
                <d:prop>
                    <d:getetag/>
                    <c:calendar-data/>
                </d:prop>
                <c:filter>
                    <c:comp-filter name="VCALENDAR">
                        <c:comp-filter name="VTODO"/>
                    </c:comp-filter>
                </c:filter>
            </c:calendar-query>""".trimIndent()

        private val REPORT_VEVENTS = """<?xml version="1.0" encoding="utf-8"?>
            <c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
                <d:prop>
                    <d:getetag/>
                    <c:calendar-data/>
                </d:prop>
                <c:filter>
                    <c:comp-filter name="VCALENDAR">
                        <c:comp-filter name="VEVENT"/>
                    </c:comp-filter>
                </c:filter>
            </c:calendar-query>""".trimIndent()
    }
}

class SyncTokenExpiredException : Exception("sync-token-expired")
