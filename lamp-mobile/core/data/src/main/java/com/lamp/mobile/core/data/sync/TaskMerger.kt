package com.lamp.mobile.core.data.sync

import com.lamp.mobile.core.model.Task

/**
 * 3-way field-level merge matching desktop's src/sync/merge.rs.
 * Server-authoritative tiebreak: when both changed to different values, remote wins.
 */
object TaskMerger {

    fun mergeTasks(local: Task, remote: Task, base: Task): Task {
        return local.copy(
            title = merge(local.title, remote.title, base.title),
            state = merge(local.state, remote.state, base.state),
            priority = merge(local.priority, remote.priority, base.priority),
            contexts = merge(local.contexts, remote.contexts, base.contexts),
            scheduled = merge(local.scheduled, remote.scheduled, base.scheduled),
            deadline = merge(local.deadline, remote.deadline, base.deadline),
            notes = merge(local.notes, remote.notes, base.notes),
            project = merge(local.project, remote.project, base.project),
            waitingFor = merge(local.waitingFor, remote.waitingFor, base.waitingFor),
            esc = merge(local.esc, remote.esc, base.esc),
            delegated = merge(local.delegated, remote.delegated, base.delegated),
            followUp = merge(local.followUp, remote.followUp, base.followUp),
            completed = merge(local.completed, remote.completed, base.completed),
            recurrence = merge(local.recurrence, remote.recurrence, base.recurrence),
        )
    }

    /**
     * 3-way merge for a single field:
     * - If only local changed: keep local
     * - If only remote changed: take remote
     * - If both changed to same: keep that value
     * - If both changed differently: remote wins (server-authoritative)
     */
    private fun <T> merge(local: T, remote: T, base: T): T {
        val localChanged = local != base
        val remoteChanged = remote != base
        return when {
            !localChanged && !remoteChanged -> local  // No change
            localChanged && !remoteChanged -> local   // Only local changed
            !localChanged && remoteChanged -> remote  // Only remote changed
            local == remote -> local                  // Both changed to same value
            else -> remote                            // Both changed differently: remote wins
        }
    }
}
