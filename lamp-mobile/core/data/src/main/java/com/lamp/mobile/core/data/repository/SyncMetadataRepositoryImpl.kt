package com.lamp.mobile.core.data.repository

import com.lamp.mobile.core.database.converter.EntityMappers.toDomain
import com.lamp.mobile.core.database.converter.EntityMappers.toEntity
import com.lamp.mobile.core.database.dao.SyncMetadataDao
import com.lamp.mobile.core.model.SyncMetadata
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SyncMetadataRepositoryImpl @Inject constructor(
    private val syncMetadataDao: SyncMetadataDao,
) : SyncMetadataRepository {

    override suspend fun getByHref(href: String): SyncMetadata? =
        syncMetadataDao.getByHref(href)?.toDomain()

    override suspend fun save(metadata: SyncMetadata) =
        syncMetadataDao.upsert(metadata.toEntity())

    override suspend fun delete(href: String) =
        syncMetadataDao.deleteByHref(href)

    override suspend fun getAll(): List<SyncMetadata> =
        syncMetadataDao.getAll().map { it.toDomain() }
}
