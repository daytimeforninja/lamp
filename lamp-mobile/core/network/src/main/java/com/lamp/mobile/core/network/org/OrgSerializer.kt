package com.lamp.mobile.core.network.org

import com.lamp.mobile.core.model.Account
import com.lamp.mobile.core.model.ListItem
import com.lamp.mobile.core.model.ListKind
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale
import java.util.UUID

object OrgSerializer {

    private val DATETIME_FMT = DateTimeFormatter.ofPattern("yyyy-MM-dd EEE HH:mm", Locale.US)
    private val DATE_FMT = DateTimeFormatter.ofPattern("yyyy-MM-dd", Locale.US)

    fun writeShoppingFile(items: List<ListItem>): String = buildString {
        appendLine("#+TITLE: Shopping")
        appendLine()
        for (item in items) {
            writeListItem(this, item)
            appendLine()
        }
    }

    fun writeAccountsFile(accounts: List<Account>): String = buildString {
        appendLine("#+TITLE: Accounts")
        appendLine()
        for (account in accounts) {
            writeAccount(this, account)
            appendLine()
        }
    }

    private fun writeListItem(sb: StringBuilder, item: ListItem) {
        if (item.done) {
            sb.appendLine("* DONE ${item.title}")
        } else {
            sb.appendLine("* ${item.title}")
        }
        sb.appendLine("  :PROPERTIES:")
        sb.appendLine("  :ID: ${item.id}")
        sb.appendLine("  :CREATED: [${item.created.format(DATETIME_FMT)}]")
        sb.appendLine("  :END:")
        if (item.notes.isNotEmpty()) {
            for (line in item.notes.lines()) {
                sb.append("  ")
                sb.appendLine(line)
            }
        }
    }

    private fun writeAccount(sb: StringBuilder, account: Account) {
        sb.appendLine("* ${account.name}")
        sb.appendLine("  :PROPERTIES:")
        sb.appendLine("  :ID: ${account.id}")
        if (account.url.isNotEmpty()) {
            sb.appendLine("  :URL: ${account.url}")
        }
        account.lastChecked?.let {
            sb.appendLine("  :LAST_CHECKED: [${it.format(DATE_FMT)}]")
        }
        sb.appendLine("  :END:")
        if (account.notes.isNotEmpty()) {
            for (line in account.notes.lines()) {
                sb.append("  ")
                sb.appendLine(line)
            }
        }
    }

    fun parseShoppingItems(content: String): List<ListItem> {
        val headings = parseOrgHeadings(content)
        return headings.map { h ->
            ListItem(
                id = h.properties["ID"]?.let { parseUuid(it) } ?: UUID.randomUUID(),
                title = h.title,
                notes = h.body,
                created = h.properties["CREATED"]?.let { parseOrgDatetime(it) } ?: LocalDateTime.now(),
                done = h.state == "DONE",
                kind = ListKind.SHOPPING,
            )
        }
    }

    fun parseAccounts(content: String): List<Account> {
        val headings = parseOrgHeadings(content)
        return headings.map { h ->
            Account(
                id = h.properties["ID"]?.let { parseUuid(it) } ?: UUID.randomUUID(),
                name = h.title,
                url = h.properties["URL"] ?: "",
                notes = h.body,
                lastChecked = h.properties["LAST_CHECKED"]?.let { parseOrgDate(it) },
            )
        }
    }

    private data class OrgHeading(
        val title: String,
        val state: String?,
        val properties: Map<String, String>,
        val body: String,
    )

    private fun parseOrgHeadings(content: String): List<OrgHeading> {
        val headings = mutableListOf<OrgHeading>()
        val lines = content.lines()
        var i = 0

        while (i < lines.size) {
            val line = lines[i]
            if (line.startsWith("* ")) {
                val (state, title) = parseHeadlineLine(line.substring(2))
                i++

                // Parse properties drawer
                val props = mutableMapOf<String, String>()
                if (i < lines.size && lines[i].trim() == ":PROPERTIES:") {
                    i++
                    while (i < lines.size && lines[i].trim() != ":END:") {
                        val propLine = lines[i].trim()
                        val match = Regex("^:([A-Za-z_]+):\\s*(.*)$").find(propLine)
                        if (match != null) {
                            props[match.groupValues[1]] = match.groupValues[2].trim()
                        }
                        i++
                    }
                    if (i < lines.size) i++ // skip :END:
                }

                // Collect body lines (indented, until next heading or EOF)
                val bodyLines = mutableListOf<String>()
                while (i < lines.size && !lines[i].startsWith("* ")) {
                    val bodyLine = lines[i]
                    // Strip 2-space indent if present
                    bodyLines.add(if (bodyLine.startsWith("  ")) bodyLine.substring(2) else bodyLine)
                    i++
                }
                // Trim trailing blank lines
                while (bodyLines.isNotEmpty() && bodyLines.last().isBlank()) {
                    bodyLines.removeAt(bodyLines.size - 1)
                }

                headings.add(OrgHeading(title, state, props, bodyLines.joinToString("\n")))
            } else {
                i++
            }
        }

        return headings
    }

    private fun parseHeadlineLine(text: String): Pair<String?, String> {
        val states = listOf("TODO", "NEXT", "DONE", "WAITING", "SOMEDAY", "CANCELLED")
        for (state in states) {
            if (text.startsWith("$state ")) {
                return state to text.substring(state.length + 1).trim()
            }
        }
        return null to text.trim()
    }

    private fun parseUuid(s: String): UUID? = try {
        UUID.fromString(s.trim())
    } catch (_: Exception) { null }

    private fun parseOrgDatetime(s: String): LocalDateTime? = try {
        val cleaned = s.trim().trimStart('[').trimEnd(']')
        LocalDateTime.parse(cleaned, DATETIME_FMT)
    } catch (_: Exception) {
        try {
            val cleaned = s.trim().trimStart('[').trimEnd(']')
            LocalDateTime.parse(cleaned, DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm", Locale.US))
        } catch (_: Exception) { null }
    }

    private fun parseOrgDate(s: String): LocalDate? = try {
        val cleaned = s.trim().trimStart('[').trimEnd(']')
        LocalDate.parse(cleaned, DATE_FMT)
    } catch (_: Exception) { null }
}
