package com.lamp.mobile.core.model

import java.util.UUID

sealed class SyncConflict {
    abstract val id: UUID

    data class StateMismatch(
        override val id: UUID = UUID.randomUUID(),
        val localTask: Task,
        val remoteTask: Task,
    ) : SyncConflict()

    data class RemoteOnly(
        override val id: UUID = UUID.randomUUID(),
        val remoteTask: Task,
    ) : SyncConflict()

    data class LocalOnly(
        override val id: UUID = UUID.randomUUID(),
        val localTask: Task,
    ) : SyncConflict()
}
