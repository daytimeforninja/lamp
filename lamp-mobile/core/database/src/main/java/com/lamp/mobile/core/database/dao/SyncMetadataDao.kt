package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.SyncMetadataEntity

@Dao
interface SyncMetadataDao {
    @Query("SELECT * FROM sync_metadata WHERE calendarHref = :href")
    suspend fun getByHref(href: String): SyncMetadataEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: SyncMetadataEntity)

    @Query("DELETE FROM sync_metadata WHERE calendarHref = :href")
    suspend fun deleteByHref(href: String)

    @Query("SELECT * FROM sync_metadata")
    suspend fun getAll(): List<SyncMetadataEntity>
}
