package com.lamp.mobile.core.network.carddav

import com.lamp.mobile.core.model.Contact
import com.lamp.mobile.core.model.ContactCategory
import com.lamp.mobile.core.network.MethodPreservingRedirectInterceptor
import com.lamp.mobile.core.network.caldav.WebDavXmlParser
import com.lamp.mobile.core.network.upgradeToHttps
import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.auth.*
import io.ktor.client.plugins.auth.providers.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import java.net.URL
import java.util.UUID

class CardDavClient(
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
                        username = this@CardDavClient.username,
                        password = this@CardDavClient.password,
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

    suspend fun fetchContacts(): Result<List<Pair<String, Contact>>> = runCatching {
        val principal = findPrincipal()
        val homeSet = findAddressbookHomeSet(principal)
        val addressbooks = listAddressbooks(homeSet)

        val allContacts = mutableListOf<Pair<String, Contact>>()
        for (ab in addressbooks) {
            allContacts.addAll(fetchAddressbookContacts(ab))
        }
        allContacts
    }

    private suspend fun findPrincipal(): String {
        val body = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:">
                <d:prop><d:current-user-principal/></d:prop>
            </d:propfind>""".trimIndent()

        // Try .well-known first (RFC 6764) — redirects followed by interceptor
        try {
            val xml = propfind(resolveUrl("/.well-known/carddav"), 0, body)
            val principal = WebDavXmlParser.extractHref(xml, "current-user-principal")
            if (principal != null) return principal
        } catch (_: Exception) { /* .well-known might not exist */ }

        // Fallback to PROPFIND on base URL
        try {
            val xml = propfind(baseUrl, 0, body)
            val principal = WebDavXmlParser.extractHref(xml, "current-user-principal")
            if (principal != null) return principal
        } catch (_: Exception) { /* ignore */ }

        return "/dav/principals/user/$username/"
    }

    private suspend fun findAddressbookHomeSet(principal: String): String {
        val body = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
                <d:prop><card:addressbook-home-set/></d:prop>
            </d:propfind>""".trimIndent()

        val xml = propfind(resolveUrl(principal), 0, body)
        return WebDavXmlParser.extractHref(xml, "addressbook-home-set")
            ?: throw Exception("No addressbook-home-set found")
    }

    private suspend fun listAddressbooks(homeSet: String): List<String> {
        val body = """<?xml version="1.0" encoding="utf-8"?>
            <d:propfind xmlns:d="DAV:">
                <d:prop>
                    <d:resourcetype/>
                    <d:displayname/>
                </d:prop>
            </d:propfind>""".trimIndent()

        val xml = propfind(resolveUrl(homeSet), 1, body)
        val responses = WebDavXmlParser.parseMultistatus(xml)
        // Filter for addressbook collections (simplified - check for addressbook in resourcetype)
        return responses.map { it.href }.filter { it != homeSet }
    }

    private suspend fun fetchAddressbookContacts(href: String): List<Pair<String, Contact>> {
        val body = """<?xml version="1.0" encoding="utf-8"?>
            <card:addressbook-query xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
                <d:prop>
                    <d:getetag/>
                    <card:address-data/>
                </d:prop>
            </card:addressbook-query>""".trimIndent()

        val response = client.request(resolveUrl(href)) {
            method = HttpMethod("REPORT")
            header("Depth", "1")
            contentType(ContentType.Application.Xml)
            setBody(body)
        }

        val xml = response.bodyAsText()
        return WebDavXmlParser.parseMultistatus(xml).mapNotNull { resp ->
            val vcard = resp.calendarData ?: return@mapNotNull null
            val contact = parseVcard(vcard) ?: return@mapNotNull null
            resp.href to contact.copy(syncHref = resp.href, syncEtag = resp.etag)
        }
    }

    suspend fun findFirstAddressbook(): Result<String> = runCatching {
        val principal = findPrincipal()
        val homeSet = findAddressbookHomeSet(principal)
        val addressbooks = listAddressbooks(homeSet)
        addressbooks.firstOrNull() ?: throw Exception("No addressbook found")
    }

    suspend fun putContact(href: String, vcard: String, etag: String?): Result<String> = runCatching {
        val response = client.put(resolveUrl(href)) {
            contentType(ContentType("text", "vcard", parameters = listOf(
                HeaderValueParam("charset", "utf-8")
            )))
            if (etag != null) {
                header("If-Match", etag)
            }
            setBody(vcard)
        }
        if (response.status.value !in 200..299) {
            throw Exception("PUT contact failed: ${response.status}")
        }
        response.headers["ETag"] ?: ""
    }

    suspend fun createContact(addressbookHref: String, vcard: String, uid: String): Result<Pair<String, String>> = runCatching {
        val href = "${addressbookHref.trimEnd('/')}/$uid.vcf"
        val response = client.put(resolveUrl(href)) {
            contentType(ContentType("text", "vcard", parameters = listOf(
                HeaderValueParam("charset", "utf-8")
            )))
            header("If-None-Match", "*")
            setBody(vcard)
        }
        if (response.status.value !in 200..299) {
            throw Exception("CREATE contact failed: ${response.status}")
        }
        val newEtag = response.headers["ETag"] ?: ""
        href to newEtag
    }

    suspend fun deleteContact(href: String): Result<Unit> = runCatching {
        val response = client.delete(resolveUrl(href))
        if (response.status.value !in listOf(200, 204, 404)) {
            throw Exception("DELETE failed: ${response.status}")
        }
    }

    private suspend fun propfind(url: String, depth: Int, body: String): String {
        val response = client.request(url) {
            method = HttpMethod("PROPFIND")
            header("Depth", depth.toString())
            contentType(ContentType.Application.Xml)
            setBody(body)
        }
        return response.bodyAsText()
    }

    fun close() = client.close()

    companion object {
        /**
         * Serialize a Contact to vCard 3.0 format (RFC 2426).
         */
        fun toVcard(contact: Contact): String = buildString {
            appendLine("BEGIN:VCARD")
            appendLine("VERSION:3.0")
            appendLine("FN:${contact.name}")
            contact.email?.let { appendLine("EMAIL:$it") }
            contact.phone?.let { appendLine("TEL:$it") }
            contact.website?.let { appendLine("URL:$it") }
            contact.signal?.let { appendLine("X-SIGNAL:$it") }
            contact.preferredMethod?.let { appendLine("X-PREFERRED-METHOD:$it") }
            val cat = if (contact.category == ContactCategory.SERVICE) "Service" else "Personal"
            appendLine("CATEGORIES:$cat")
            appendLine("END:VCARD")
        }

        /**
         * Parse a vCard string into a Contact.
         * Handles FN, EMAIL, TEL, URL, X-SIGNAL, X-PREFERRED-METHOD, CATEGORIES.
         * Matches desktop's parse_vcard logic.
         */
        fun parseVcard(vcard: String): Contact? {
            // Unfold continuation lines (RFC 6350)
            val unfolded = vcard
                .replace("\r\n ", "")
                .replace("\r\n\t", "")
                .replace("\n ", "")
                .replace("\n\t", "")

            var name: String? = null
            var email: String? = null
            var phone: String? = null
            var website: String? = null
            var signal: String? = null
            var preferredMethod: String? = null
            var category = ContactCategory.PERSONAL

            for (line in unfolded.lines()) {
                val colonIdx = line.indexOf(':')
                if (colonIdx < 0) continue
                val keyPart = line.substring(0, colonIdx).uppercase()
                val value = line.substring(colonIdx + 1).trim()
                val key = keyPart.split(';').first()

                when (key) {
                    "FN" -> name = value
                    "EMAIL" -> if (email == null) email = value
                    "TEL" -> if (phone == null) phone = value
                    "URL" -> if (website == null) website = value
                    "X-SIGNAL" -> signal = value
                    "X-PREFERRED-METHOD" -> preferredMethod = value
                    "CATEGORIES" -> {
                        if (value.split(",").any { it.trim().equals("Service", ignoreCase = true) }) {
                            category = ContactCategory.SERVICE
                        }
                    }
                }
            }

            if (name.isNullOrBlank()) return null

            return Contact(
                name = name,
                email = email,
                phone = phone,
                website = website,
                signal = signal,
                preferredMethod = preferredMethod,
                category = category,
            )
        }

        /**
         * Merge remote contacts into local contacts.
         * Matches desktop's merge_contacts logic:
         * - Match by sync_href first, then by name
         * - Remote wins for communication fields
         * - Local preserves last_contacted
         */
        fun mergeContacts(local: List<Contact>, remote: List<Pair<String, Contact>>): List<Contact> {
            val localByHref = local.filter { it.syncHref != null }.associateBy { it.syncHref!! }.toMutableMap()
            val localByName = local.associateBy { it.name.lowercase() }.toMutableMap()
            val matched = mutableSetOf<UUID>()
            val result = mutableListOf<Contact>()

            for ((href, remoteContact) in remote) {
                // Match by href first
                val byHref = localByHref[href]
                if (byHref != null) {
                    matched.add(byHref.id)
                    result.add(byHref.copy(
                        name = remoteContact.name,
                        email = remoteContact.email ?: byHref.email,
                        phone = remoteContact.phone ?: byHref.phone,
                        website = remoteContact.website ?: byHref.website,
                        signal = remoteContact.signal ?: byHref.signal,
                        preferredMethod = remoteContact.preferredMethod ?: byHref.preferredMethod,
                        category = remoteContact.category,
                        syncHref = href,
                    ))
                    continue
                }

                // Match by name
                val byName = localByName[remoteContact.name.lowercase()]
                if (byName != null && byName.id !in matched) {
                    matched.add(byName.id)
                    result.add(byName.copy(
                        email = remoteContact.email ?: byName.email,
                        phone = remoteContact.phone ?: byName.phone,
                        website = remoteContact.website ?: byName.website,
                        signal = remoteContact.signal ?: byName.signal,
                        preferredMethod = remoteContact.preferredMethod ?: byName.preferredMethod,
                        category = remoteContact.category,
                        syncHref = href,
                    ))
                    continue
                }

                // New remote contact
                result.add(remoteContact.copy(syncHref = href))
            }

            // Keep unmatched local contacts
            for (localContact in local) {
                if (localContact.id !in matched) {
                    result.add(localContact)
                }
            }

            return result.sortedBy { it.name }
        }
    }
}
