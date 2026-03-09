package com.lamp.mobile.core.network.ical

import com.lamp.mobile.core.model.CalendarEvent
import com.lamp.mobile.core.model.EventStatus
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.LocalTime
import java.util.UUID

object VeventConverter {

    fun vcalendarToEvent(ical: String, calendarHref: String, calendarName: String): CalendarEvent? {
        val unfolded = ICalHelpers.unfoldLines(ical)
        val lines = unfolded.lines()

        var inVevent = false
        var uid: String? = null
        var summary: String? = null
        var dtstart: String? = null
        var dtend: String? = null
        var allDay = false
        var location: String? = null
        var description: String? = null
        var status: String? = null

        for (line in lines) {
            val trimmed = line.trim()
            if (trimmed == "BEGIN:VEVENT") { inVevent = true; continue }
            if (trimmed == "END:VEVENT") break
            if (!inVevent) continue

            // Handle VALUE=DATE for all-day detection
            if (trimmed.startsWith("DTSTART;VALUE=DATE:")) {
                dtstart = trimmed.substringAfter(":")
                allDay = true
                continue
            }
            if (trimmed.startsWith("DTEND;VALUE=DATE:")) {
                dtend = trimmed.substringAfter(":")
                continue
            }

            val parsed = ICalHelpers.parseIcalLine(trimmed) ?: continue
            val (key, value) = parsed

            when (key.uppercase()) {
                "UID" -> uid = value
                "SUMMARY" -> summary = ICalHelpers.unescapeText(value)
                "DTSTART" -> if (dtstart == null) dtstart = value
                "DTEND" -> if (dtend == null) dtend = value
                "LOCATION" -> location = ICalHelpers.unescapeText(value)
                "DESCRIPTION" -> description = ICalHelpers.unescapeText(value)
                "STATUS" -> status = value
            }
        }

        if (summary.isNullOrBlank()) return null

        val start = if (allDay) {
            ICalHelpers.parseIcalDate(dtstart ?: return null)?.atStartOfDay()
        } else {
            dtstart?.let { ICalHelpers.parseIcalDatetime(it) }
        } ?: return null

        val end = if (allDay) {
            ICalHelpers.parseIcalDate(dtend ?: dtstart!!)?.atStartOfDay()
        } else {
            dtend?.let { ICalHelpers.parseIcalDatetime(it) }
        } ?: start.plusHours(1)

        val eventId = uid?.let {
            try { UUID.fromString(it) }
            catch (_: Exception) { ICalHelpers.uuidV5(ICalHelpers.CALDAV_UUID_NAMESPACE, it) }
        } ?: UUID.randomUUID()

        return CalendarEvent(
            id = eventId,
            title = summary,
            start = start,
            end = end,
            allDay = allDay,
            location = location ?: "",
            description = description ?: "",
            status = EventStatus.fromIcal(status ?: "CONFIRMED"),
            calendarHref = calendarHref,
            calendarName = calendarName,
            syncUid = uid,
        )
    }

    fun eventToVcalendar(event: CalendarEvent): String {
        val lines = mutableListOf<String>()
        lines.add("BEGIN:VCALENDAR")
        lines.add("VERSION:2.0")
        lines.add("PRODID:-//Lamp GTD//EN")
        lines.add("BEGIN:VEVENT")
        lines.add("UID:${event.id}")
        lines.add(ICalHelpers.foldLine("SUMMARY:${ICalHelpers.escapeText(event.title)}"))

        if (event.allDay) {
            lines.add("DTSTART;VALUE=DATE:${ICalHelpers.formatDate(event.start.toLocalDate())}")
            lines.add("DTEND;VALUE=DATE:${ICalHelpers.formatDate(event.end.toLocalDate())}")
        } else {
            lines.add("DTSTART:${ICalHelpers.formatDatetime(event.start)}")
            lines.add("DTEND:${ICalHelpers.formatDatetime(event.end)}")
        }

        if (event.location.isNotBlank()) {
            lines.add(ICalHelpers.foldLine("LOCATION:${ICalHelpers.escapeText(event.location)}"))
        }
        if (event.description.isNotBlank()) {
            lines.add(ICalHelpers.foldLine("DESCRIPTION:${ICalHelpers.escapeText(event.description)}"))
        }

        lines.add("STATUS:${event.status.name}")
        lines.add("LAST-MODIFIED:${ICalHelpers.formatDatetime(LocalDateTime.now())}")
        lines.add("END:VEVENT")
        lines.add("END:VCALENDAR")

        return lines.joinToString("\r\n")
    }

    fun veventHref(calendarHref: String, eventUid: String): String =
        "${calendarHref.trimEnd('/')}/$eventUid.ics"
}
