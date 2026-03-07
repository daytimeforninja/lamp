package com.lamp.mobile.core.network.caldav

import com.lamp.mobile.core.model.CalendarInfo
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

    // --- Discovery ---

    suspend fun discoverCalendars(): Result<List<CalendarInfo>> = runCatching {
        val principal = findPrincipal()
        val homeSet = findCalendarHomeSet(principal)
        listCalendars(homeSet)
    }

    private suspend fun findPrincipal(): String {
        // Try .well-known first (RFC 6764)
        val wellKnownBody = propfind("$baseUrl/.well-known/caldav", 0, PROPFIND_PRINCIPAL)
        var principal = WebDavXmlParser.extractHref(wellKnownBody, "current-user-principal")
        if (principal == null) {
            val rootBody = propfind(baseUrl, 0, PROPFIND_PRINCIPAL)
            principal = WebDavXmlParser.extractHref(rootBody, "current-user-principal")
        }
        // Fastmail fallback
        return principal ?: "/dav/principals/user/$username/"
    }

    private suspend fun findCalendarHomeSet(principal: String): String {
        val body = propfind(resolveUrl(principal), 0, PROPFIND_HOME_SET)
        return WebDavXmlParser.extractHref(body, "calendar-home-set")
            ?: throw Exception("No calendar-home-set found")
    }

    private suspend fun listCalendars(homeSet: String): List<CalendarInfo> {
        val body = propfind(resolveUrl(homeSet), 1, PROPFIND_CALENDARS)
        return WebDavXmlParser.parseCalendarListing(body)
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

    // --- Helpers ---

    private suspend fun propfind(url: String, depth: Int, body: String): String {
        val response = client.request(url) {
            method = HttpMethod("PROPFIND")
            header("Depth", depth.toString())
            contentType(ContentType.Application.Xml)
            setBody(body)
        }
        return response.bodyAsText()
    }

    private suspend fun report(url: String, body: String): String {
        val response = client.request(url) {
            method = HttpMethod("REPORT")
            header("Depth", "1")
            contentType(ContentType.Application.Xml)
            setBody(body)
        }
        return response.bodyAsText()
    }

    private fun extractUid(ical: String): String? =
        ical.lineSequence().find { it.startsWith("UID:") }?.substringAfter("UID:")

    fun close() = client.close()

    companion object {
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
