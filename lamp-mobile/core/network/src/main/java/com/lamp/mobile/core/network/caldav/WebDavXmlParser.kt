package com.lamp.mobile.core.network.caldav

import com.lamp.mobile.core.model.CalendarInfo
import org.xmlpull.v1.XmlPullParser
import org.xmlpull.v1.XmlPullParserFactory

/**
 * Parses WebDAV/CalDAV XML responses (multistatus, propfind).
 */
object WebDavXmlParser {

    private const val NS_DAV = "DAV:"
    private const val NS_CALDAV = "urn:ietf:params:xml:ns:caldav"
    private const val NS_CARDDAV = "urn:ietf:params:xml:ns:carddav"

    data class MultistatusResponse(
        val href: String,
        val etag: String?,
        val calendarData: String?,
        val status: String?,
    )

    fun parseMultistatus(xml: String): List<MultistatusResponse> {
        val responses = mutableListOf<MultistatusResponse>()
        val parser = createParser(xml)

        var inResponse = false
        var href: String? = null
        var etag: String? = null
        var calendarData: String? = null
        var addressData: String? = null
        var status: String? = null
        var currentTag = ""

        while (parser.eventType != XmlPullParser.END_DOCUMENT) {
            when (parser.eventType) {
                XmlPullParser.START_TAG -> {
                    currentTag = parser.name
                    if (currentTag == "response") {
                        inResponse = true
                        href = null; etag = null; calendarData = null; addressData = null; status = null
                    }
                }
                XmlPullParser.TEXT -> if (inResponse) {
                    val text = parser.text?.trim() ?: ""
                    if (text.isNotEmpty()) when (currentTag) {
                        "href" -> href = text
                        "getetag" -> etag = text.trim('"')
                        "calendar-data" -> calendarData = text
                        "address-data" -> addressData = text
                        "status" -> status = text
                    }
                }
                XmlPullParser.END_TAG -> {
                    if (parser.name == "response" && inResponse) {
                        href?.let { h ->
                            responses.add(MultistatusResponse(
                                href = h,
                                etag = etag,
                                calendarData = calendarData ?: addressData,
                                status = status,
                            ))
                        }
                        inResponse = false
                    }
                    currentTag = ""
                }
            }
            parser.next()
        }
        return responses
    }

    fun parseCalendarListing(xml: String): List<CalendarInfo> {
        val calendars = mutableListOf<CalendarInfo>()
        val responses = parseMultistatus(xml)
        // We need more detailed parsing for calendar info
        // Reparse with full detail
        val parser = createParser(xml)

        var inResponse = false
        var href: String? = null
        var displayName: String? = null
        var isCalendar = false
        var supportsVtodo = false
        var supportsVevent = false
        var currentTag = ""
        var depth = 0

        while (parser.eventType != XmlPullParser.END_DOCUMENT) {
            when (parser.eventType) {
                XmlPullParser.START_TAG -> {
                    depth++
                    currentTag = parser.name
                    when (currentTag) {
                        "response" -> {
                            inResponse = true
                            href = null; displayName = null; isCalendar = false
                            supportsVtodo = false; supportsVevent = false
                        }
                        "calendar" -> if (inResponse) isCalendar = true
                        "comp" -> if (inResponse) {
                            val name = parser.getAttributeValue(null, "name")
                            when (name) {
                                "VTODO" -> supportsVtodo = true
                                "VEVENT" -> supportsVevent = true
                            }
                        }
                    }
                }
                XmlPullParser.TEXT -> if (inResponse) {
                    val text = parser.text?.trim() ?: ""
                    if (text.isNotEmpty()) when (currentTag) {
                        "href" -> href = text
                        "displayname" -> displayName = text
                    }
                }
                XmlPullParser.END_TAG -> {
                    depth--
                    if (parser.name == "response" && inResponse) {
                        if (isCalendar && href != null) {
                            calendars.add(CalendarInfo(
                                href = href!!,
                                displayName = displayName ?: href!!,
                                supportsVtodo = supportsVtodo,
                                supportsVevent = supportsVevent,
                            ))
                        }
                        inResponse = false
                    }
                    currentTag = ""
                }
            }
            parser.next()
        }
        return calendars
    }

    fun extractHref(xml: String, propertyName: String): String? {
        val parser = createParser(xml)
        var inProperty = false
        var currentTag = ""

        while (parser.eventType != XmlPullParser.END_DOCUMENT) {
            when (parser.eventType) {
                XmlPullParser.START_TAG -> {
                    currentTag = parser.name
                    if (currentTag == propertyName) inProperty = true
                }
                XmlPullParser.TEXT -> {
                    if (inProperty && currentTag == "href") {
                        return parser.text?.trim()
                    }
                }
                XmlPullParser.END_TAG -> {
                    if (parser.name == propertyName) inProperty = false
                    currentTag = ""
                }
            }
            parser.next()
        }
        return null
    }

    fun extractSyncToken(xml: String): String? {
        val parser = createParser(xml)
        var currentTag = ""
        while (parser.eventType != XmlPullParser.END_DOCUMENT) {
            when (parser.eventType) {
                XmlPullParser.START_TAG -> currentTag = parser.name
                XmlPullParser.TEXT -> {
                    if (currentTag == "sync-token") return parser.text?.trim()
                }
                XmlPullParser.END_TAG -> currentTag = ""
            }
            parser.next()
        }
        return null
    }

    private fun createParser(xml: String): XmlPullParser {
        if (xml.isBlank()) {
            throw Exception("Server returned empty response")
        }
        val factory = XmlPullParserFactory.newInstance()
        factory.isNamespaceAware = true
        val parser = factory.newPullParser()
        parser.setInput(xml.reader())
        return parser
    }
}
