package com.lamp.mobile.core.database.dao

import androidx.room.*
import com.lamp.mobile.core.database.entity.ListItemEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface ListItemDao {
    @Query("SELECT * FROM list_items WHERE kind = :kind ORDER BY created DESC")
    fun observeByKind(kind: String): Flow<List<ListItemEntity>>

    @Query("SELECT * FROM list_items WHERE id = :id")
    suspend fun getById(id: String): ListItemEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(entity: ListItemEntity)

    @Query("DELETE FROM list_items WHERE id = :id")
    suspend fun deleteById(id: String)

    @Query("SELECT * FROM list_items WHERE kind = :kind")
    suspend fun getByKind(kind: String): List<ListItemEntity>
}
