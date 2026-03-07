package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.model.SyncMetadata

interface SyncMetadataRepository {
    suspend fun getByHref(href: String): SyncMetadata?
    suspend fun save(metadata: SyncMetadata)
    suspend fun delete(href: String)
    suspend fun getAll(): List<SyncMetadata>
}
