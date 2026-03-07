package com.lamp.mobile.core.model

import org.junit.Assert.*
import org.junit.Test
import java.util.UUID

class LinkTargetTest {

    @Test
    fun `toOrg and fromOrg roundtrip`() {
        val id = UUID.fromString("550e8400-e29b-41d4-a716-446655440000")
        val targets = listOf(
            LinkTarget.NoteLink(id),
            LinkTarget.TaskLink(id),
            LinkTarget.ProjectLink(id),
            LinkTarget.ContactLink(id),
            LinkTarget.AccountLink(id),
            LinkTarget.MediaItemLink(id),
            LinkTarget.ShoppingItemLink(id),
        )
        for (target in targets) {
            val org = target.toOrg()
            val parsed = LinkTarget.fromOrg(org)
            assertEquals(target, parsed)
        }
    }

    @Test
    fun `fromOrg returns null for invalid input`() {
        assertNull(LinkTarget.fromOrg("invalid"))
        assertNull(LinkTarget.fromOrg("note:not-a-uuid"))
        assertNull(LinkTarget.fromOrg(""))
    }
}
