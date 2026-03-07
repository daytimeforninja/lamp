package com.lamp.mobile.core.model

import java.util.UUID

sealed class LinkTarget {
    abstract val targetId: UUID

    data class NoteLink(override val targetId: UUID) : LinkTarget()
    data class TaskLink(override val targetId: UUID) : LinkTarget()
    data class ProjectLink(override val targetId: UUID) : LinkTarget()
    data class ContactLink(override val targetId: UUID) : LinkTarget()
    data class AccountLink(override val targetId: UUID) : LinkTarget()
    data class MediaItemLink(override val targetId: UUID) : LinkTarget()
    data class ShoppingItemLink(override val targetId: UUID) : LinkTarget()

    fun toOrg(): String {
        val type = when (this) {
            is NoteLink -> "note"
            is TaskLink -> "task"
            is ProjectLink -> "project"
            is ContactLink -> "contact"
            is AccountLink -> "account"
            is MediaItemLink -> "media"
            is ShoppingItemLink -> "shopping"
        }
        return "$type:$targetId"
    }

    companion object {
        fun fromOrg(s: String): LinkTarget? {
            val parts = s.split(":", limit = 2)
            if (parts.size != 2) return null
            val uuid = try { UUID.fromString(parts[1]) } catch (_: Exception) { return null }
            return when (parts[0]) {
                "note" -> NoteLink(uuid)
                "task" -> TaskLink(uuid)
                "project" -> ProjectLink(uuid)
                "contact" -> ContactLink(uuid)
                "account" -> AccountLink(uuid)
                "media" -> MediaItemLink(uuid)
                "shopping" -> ShoppingItemLink(uuid)
                else -> null
            }
        }
    }
}
