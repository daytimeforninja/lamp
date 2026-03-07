package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.ContactEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface ContactDao {
    @Query("SELECT * FROM contacts ORDER BY name ASC")
    fun observeAll(): Flow<List<ContactEntity>>

    @Query("SELECT * FROM contacts WHERE id = :id")
    suspend fun getById(id: String): ContactEntity?

    @Query("SELECT * FROM contacts WHERE syncHref = :href")
    suspend fun getByHref(href: String): ContactEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: ContactEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertAll(entities: List<ContactEntity>)

    @Query("DELETE FROM contacts WHERE id = :id")
    suspend fun deleteById(id: String)

    @Query("SELECT * FROM contacts WHERE syncDeleted = 0")
    suspend fun getAll(): List<ContactEntity>

    @Query("SELECT * FROM contacts WHERE syncDirty = 1 AND syncDeleted = 0")
    suspend fun getDirty(): List<ContactEntity>

    @Query("SELECT * FROM contacts WHERE syncDeleted = 1")
    suspend fun getDeleted(): List<ContactEntity>

    @Query("UPDATE contacts SET syncDirty = 0, syncHref = :href, syncEtag = :etag WHERE id = :id")
    suspend fun markSynced(id: String, href: String, etag: String?)

    @Query("UPDATE contacts SET syncDeleted = 1 WHERE id = :id")
    suspend fun markDeleted(id: String)
}
