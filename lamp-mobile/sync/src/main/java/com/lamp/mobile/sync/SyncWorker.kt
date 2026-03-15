package com.lamp.mobile.sync

import android.accounts.Account
import android.accounts.AccountManager
import android.content.ContentResolver
import android.content.Context
import android.os.Bundle
import android.provider.ContactsContract
import android.util.Log
import androidx.hilt.work.HiltWorker
import androidx.work.*
import com.lamp.mobile.core.data.sync.SyncEngine
import dagger.assisted.Assisted
import dagger.assisted.AssistedInject
import java.util.concurrent.TimeUnit

@HiltWorker
class SyncWorker @AssistedInject constructor(
    @Assisted context: Context,
    @Assisted workerParams: WorkerParameters,
    private val syncEngine: SyncEngine,
) : CoroutineWorker(context, workerParams) {

    override suspend fun doWork(): Result {
        return try {
            Log.d(TAG, "Starting sync...")
            val result = syncEngine.syncAll()
            Log.d(TAG, "Sync complete: ${result.tasksUploaded} up, ${result.tasksDownloaded} down, ${result.conflicts.size} conflicts, ${result.errors.size} errors")
            if (result.errors.isNotEmpty()) {
                Log.w(TAG, "Sync errors: ${result.errors.joinToString("; ")}")
            }
            if (result.conflicts.isNotEmpty()) {
                Log.w(TAG, "Sync conflicts: ${result.conflicts.size}")
            }
            if (result.contactsSynced > 0) {
                requestSystemContactsSync(applicationContext)
            }
            Result.success(workDataOf(
                "tasks_uploaded" to result.tasksUploaded,
                "tasks_downloaded" to result.tasksDownloaded,
                "events_downloaded" to result.eventsDownloaded,
                "contacts_synced" to result.contactsSynced,
                "notes_synced" to result.notesSynced,
                "conflict_count" to result.conflicts.size,
                "error_count" to result.errors.size,
            ))
        } catch (e: Exception) {
            Log.e(TAG, "Sync failed", e)
            if (runAttemptCount < 3) Result.retry() else Result.failure()
        }
    }

    private fun requestSystemContactsSync(context: Context) {
        try {
            val accountManager = AccountManager.get(context)
            val account = Account(ACCOUNT_NAME, ACCOUNT_TYPE)
            if (accountManager.getAccountsByType(ACCOUNT_TYPE).isEmpty()) {
                val added = accountManager.addAccountExplicitly(account, null, null)
                if (added) {
                    ContentResolver.setIsSyncable(account, ContactsContract.AUTHORITY, 1)
                    ContentResolver.setSyncAutomatically(account, ContactsContract.AUTHORITY, true)
                    Log.d(TAG, "Created Lamp system account for contacts sync")
                } else {
                    Log.w(TAG, "Failed to create system account")
                    return
                }
            }
            ContentResolver.requestSync(account, ContactsContract.AUTHORITY, Bundle.EMPTY)
            Log.d(TAG, "Requested system contacts sync")
        } catch (e: Exception) {
            Log.e(TAG, "System contacts sync failed", e)
        }
    }

    companion object {
        const val TAG = "LampSync"
        private const val ACCOUNT_NAME = "Lamp"
        private const val ACCOUNT_TYPE = "com.lamp.mobile"
        const val PERIODIC_WORK_NAME = "lamp_periodic_sync"
        const val ONE_TIME_WORK_NAME = "lamp_one_time_sync"

        fun enqueuePeriodicSync(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiredNetworkType(NetworkType.CONNECTED)
                .build()

            val request = PeriodicWorkRequestBuilder<SyncWorker>(15, TimeUnit.MINUTES)
                .setConstraints(constraints)
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 1, TimeUnit.MINUTES)
                .build()

            WorkManager.getInstance(context)
                .enqueueUniquePeriodicWork(
                    PERIODIC_WORK_NAME,
                    ExistingPeriodicWorkPolicy.KEEP,
                    request,
                )
        }

        fun enqueueOneTimeSync(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiredNetworkType(NetworkType.CONNECTED)
                .build()

            val request = OneTimeWorkRequestBuilder<SyncWorker>()
                .setConstraints(constraints)
                .build()

            WorkManager.getInstance(context)
                .enqueueUniqueWork(
                    ONE_TIME_WORK_NAME,
                    ExistingWorkPolicy.REPLACE,
                    request,
                )
        }
    }
}
