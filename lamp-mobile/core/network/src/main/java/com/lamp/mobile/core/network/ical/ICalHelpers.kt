package com.lamp.mobile.core.network.ical

import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter

object ICalHelpers {
    fun formatDate(date: LocalDate): String =
        date.format(DateTimeFormatter.BASIC_ISO_DATE) // YYYYMMDD

    fun formatDatetime(dt: LocalDateTime): String {
        val utc = dt.atZone(ZoneId.systemDefault()).withZoneSameInstant(ZoneOffset.UTC)
        return utc.format(DateTimeFormatter.ofPattern("yyyyMMdd'T'HHmmss'Z'"))
    }

    fun parseIcalDate(s: String): LocalDate? = try {
        LocalDate.parse(s.take(8), DateTimeFormatter.BASIC_ISO_DATE)
    } catch (_: Exception) { null }

    fun parseIcalDatetime(s: String): LocalDateTime? = try {
        val clean = s.trim()
        if (clean.endsWith("Z") || clean.endsWith("z")) {
            val utc = LocalDateTime.parse(
                clean.dropLast(1),
                DateTimeFormatter.ofPattern("yyyyMMdd'T'HHmmss")
            )
            utc.atOffset(ZoneOffset.UTC)
                .atZoneSameInstant(ZoneId.systemDefault())
                .toLocalDateTime()
        } else {
            LocalDateTime.parse(clean, DateTimeFormatter.ofPattern("yyyyMMdd'T'HHmmss"))
        }
    } catch (_: Exception) { null }

    fun escapeText(s: String): String = s
        .replace("\\", "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")

    fun unescapeText(s: String): String = s
        .replace("\\n", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")

    fun parseIcalLine(line: String): Pair<String, String>? {
        val colonIdx = line.indexOf(':')
        if (colonIdx < 0) return null
        val keyPart = line.substring(0, colonIdx)
        val value = line.substring(colonIdx + 1)
        // Strip parameters (e.g., "DTSTART;VALUE=DATE" -> "DTSTART")
        val key = keyPart.split(';').first()
        return key to value
    }

    fun unfoldLines(input: String): String = input
        .replace("\r\n ", "")
        .replace("\r\n\t", "")
        .replace("\n ", "")
        .replace("\n\t", "")

    fun foldLine(s: String): String {
        if (s.toByteArray(Charsets.UTF_8).size <= 75) return s
        val sb = StringBuilder()
        var lineBytes = 0
        var first = true
        for (ch in s) {
            val charBytes = ch.toString().toByteArray(Charsets.UTF_8).size
            if (!first && lineBytes + charBytes > 75) {
                sb.append("\r\n ")
                lineBytes = 1 // leading space
            }
            sb.append(ch)
            lineBytes += charBytes
            first = false
        }
        return sb.toString()
    }
}
