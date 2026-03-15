package com.lamp.mobile.core.network.imap

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.io.PrintWriter
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale
import javax.net.ssl.SSLSocketFactory

data class ImapEmail(
    val uid: Long,
    val messageId: String?,
    val subject: String,
    val from: String,
    val date: LocalDateTime?,
    val bodyPreview: String,
    val bodyFull: String,
)

class ImapClient(
    private val host: String,
    private val username: String,
    private val password: String,
) {
    suspend fun fetchEmails(folderName: String): Result<List<ImapEmail>> = withContext(Dispatchers.IO) {
        runCatching {
            ImapSession(host, username, password).use { session ->
                session.select(folderName)
                val uids = session.searchAll()
                if (uids.isEmpty()) return@use emptyList()

                uids.map { uid ->
                    val headers = session.fetchHeaders(uid)
                    val body = session.fetchBody(uid)
                    val subject = headers["subject"] ?: "(no subject)"
                    val from = headers["from"]?.let { parseFrom(it) } ?: "unknown"
                    val messageId = headers["message-id"]
                    val date = headers["date"]?.let { parseDate(it) }
                    val preview = body.take(200)
                    ImapEmail(uid, messageId, subject, from, date, preview, body)
                }
            }
        }
    }

    suspend fun archiveEmail(folderName: String, uid: Long): Result<Unit> = withContext(Dispatchers.IO) {
        runCatching {
            ImapSession(host, username, password).use { session ->
                session.select(folderName)
                session.copy(uid, "Archive")
                session.addFlag(uid, "\\Deleted")
                session.expunge()
            }
        }
    }

    suspend fun testConnection(): Result<String> = withContext(Dispatchers.IO) {
        runCatching {
            ImapSession(host, username, password).use { session ->
                val folders = session.listFolders()
                "Connected, ${folders.size} folders"
            }
        }
    }

    private fun parseFrom(raw: String): String {
        // "Display Name <email@host>" → "Display Name"
        val match = Regex("^\"?([^\"<]+?)\"?\\s*<").find(raw.trim())
        return match?.groupValues?.get(1)?.trim() ?: raw.substringBefore("<").trim().ifEmpty { raw.trim() }
    }

    private fun parseDate(raw: String): LocalDateTime? {
        val formats = listOf(
            "EEE, d MMM yyyy HH:mm:ss Z",
            "EEE, d MMM yyyy HH:mm:ss z",
            "d MMM yyyy HH:mm:ss Z",
            "EEE, dd MMM yyyy HH:mm:ss Z",
        )
        for (fmt in formats) {
            try {
                val ta = DateTimeFormatter.ofPattern(fmt, Locale.US).parse(raw.trim())
                return LocalDateTime.from(java.time.ZonedDateTime.from(ta))
            } catch (_: Exception) {}
        }
        return null
    }
}

private class ImapSession(
    host: String,
    username: String,
    password: String,
) : AutoCloseable {
    private val socket = SSLSocketFactory.getDefault().createSocket(host, 993)
    private val reader = BufferedReader(InputStreamReader(socket.getInputStream(), Charsets.UTF_8))
    private val writer = PrintWriter(OutputStreamWriter(socket.getOutputStream(), Charsets.UTF_8), true)
    private var tag = 0

    init {
        // Read greeting
        readLine()
        // Login
        val loginResp = command("LOGIN \"${escape(username)}\" \"${escape(password)}\"")
        if (!loginResp.startsWith("OK")) throw Exception("Login failed: $loginResp")
    }

    fun select(folder: String) {
        val resp = command("SELECT \"${escape(folder)}\"")
        if (!resp.startsWith("OK")) throw Exception("SELECT failed: $resp")
    }

    fun searchAll(): List<Long> {
        val resp = commandWithData("UID SEARCH ALL")
        val searchLine = resp.first.find { it.contains("* SEARCH") } ?: return emptyList()
        return searchLine.substringAfter("* SEARCH").trim().split(" ")
            .filter { it.isNotBlank() }
            .mapNotNull { it.toLongOrNull() }
    }

    fun fetchHeaders(uid: Long): Map<String, String> {
        val resp = commandWithData("UID FETCH $uid (BODY.PEEK[HEADER.FIELDS (SUBJECT FROM DATE MESSAGE-ID)])")
        val raw = resp.first.joinToString("\n")
        val headers = mutableMapOf<String, String>()
        // Parse header block between { and )
        val headerBlock = raw.substringAfter("{").substringAfter("\n")
        var currentKey = ""
        var currentValue = ""
        for (line in headerBlock.lines()) {
            if (line.startsWith(")") || line.startsWith("${tag}")) break
            if (line.matches(Regex("^[A-Za-z-]+:.*"))) {
                if (currentKey.isNotEmpty()) headers[currentKey.lowercase()] = currentValue.trim()
                val colon = line.indexOf(':')
                currentKey = line.substring(0, colon)
                currentValue = line.substring(colon + 1)
            } else if (currentKey.isNotEmpty() && line.startsWith(" ") || line.startsWith("\t")) {
                currentValue += " ${line.trim()}"
            }
        }
        if (currentKey.isNotEmpty()) headers[currentKey.lowercase()] = currentValue.trim()
        return headers
    }

    fun fetchBody(uid: Long): String {
        val resp = commandWithData("UID FETCH $uid (BODY.PEEK[TEXT]<0.2000>)")
        val raw = resp.first.joinToString("\n")
        // Extract text between literal marker and closing paren
        val start = raw.indexOf("}\r\n").takeIf { it >= 0 }?.plus(3)
            ?: raw.indexOf("}\n").takeIf { it >= 0 }?.plus(2)
            ?: return ""
        val body = raw.substring(start)
        // Remove trailing ) and tag line
        return body.lines()
            .takeWhile { !it.startsWith(")") && !it.matches(Regex("^[Aa]\\d+ .*")) }
            .joinToString("\n")
            .trim()
    }

    fun copy(uid: Long, destFolder: String) {
        val resp = command("UID COPY $uid \"${escape(destFolder)}\"")
        if (!resp.startsWith("OK") && !resp.contains("TRYCREATE")) {
            throw Exception("COPY failed: $resp")
        }
        if (resp.contains("TRYCREATE")) {
            command("CREATE \"${escape(destFolder)}\"")
            val retry = command("UID COPY $uid \"${escape(destFolder)}\"")
            if (!retry.startsWith("OK")) throw Exception("COPY failed after CREATE: $retry")
        }
    }

    fun addFlag(uid: Long, flag: String) {
        command("UID STORE $uid +FLAGS ($flag)")
    }

    fun expunge() {
        command("EXPUNGE")
    }

    fun listFolders(): List<String> {
        val resp = commandWithData("LIST \"\" \"*\"")
        return resp.first
            .filter { it.startsWith("* LIST") }
            .mapNotNull { line ->
                // * LIST (\flags) "." "FolderName"
                val parts = line.split("\"")
                parts.lastOrNull()?.takeIf { it.isNotBlank() }
            }
    }

    private fun command(cmd: String): String {
        tag++
        val t = "A$tag"
        writer.println("$t $cmd")
        writer.flush()
        while (true) {
            val line = reader.readLine() ?: throw Exception("Connection closed")
            if (line.startsWith("$t ")) {
                return line.substringAfter("$t ").trim()
            }
        }
    }

    private fun commandWithData(cmd: String): Pair<List<String>, String> {
        tag++
        val t = "A$tag"
        writer.println("$t $cmd")
        writer.flush()
        val lines = mutableListOf<String>()
        while (true) {
            val line = reader.readLine() ?: throw Exception("Connection closed")
            if (line.startsWith("$t ")) {
                return Pair(lines, line.substringAfter("$t ").trim())
            }
            // Handle literal continuations {N}
            val literalMatch = Regex("\\{(\\d+)\\}$").find(line)
            if (literalMatch != null) {
                lines.add(line)
                val len = literalMatch.groupValues[1].toInt()
                val buf = CharArray(len)
                var read = 0
                while (read < len) {
                    val n = reader.read(buf, read, len - read)
                    if (n < 0) break
                    read += n
                }
                lines.add(String(buf, 0, read))
            } else {
                lines.add(line)
            }
        }
    }

    private fun readLine(): String = reader.readLine() ?: throw Exception("Connection closed")

    private fun escape(s: String) = s.replace("\\", "\\\\").replace("\"", "\\\"")

    override fun close() {
        try {
            tag++
            writer.println("A$tag LOGOUT")
            writer.flush()
        } catch (_: Exception) {}
        try { socket.close() } catch (_: Exception) {}
    }
}
