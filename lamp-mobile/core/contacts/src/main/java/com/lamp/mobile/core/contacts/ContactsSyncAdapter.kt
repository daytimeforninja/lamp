package com.lamp.mobile.core.contacts

import android.accounts.Account
import android.content.*
import android.os.Bundle
import android.provider.ContactsContract
import com.lamp.mobile.core.data.repository.ContactRepository
import com.lamp.mobile.core.model.Contact
import kotlinx.coroutines.runBlocking

/**
 * Android SyncAdapter that writes Lamp contacts into the system ContactsProvider.
 * This makes contacts available in the phone's native Contacts app, dialer, etc.
 * Groups/categories are synced as ContactsContract group memberships.
 */
class ContactsSyncAdapter(
    context: Context,
    autoInitialize: Boolean,
    private val contactRepository: ContactRepository,
) : AbstractThreadedSyncAdapter(context, autoInitialize) {

    override fun onPerformSync(
        account: Account,
        extras: Bundle,
        authority: String,
        provider: ContentProviderClient,
        syncResult: SyncResult,
    ) {
        runBlocking {
            try {
                val contacts = contactRepository.getAll()
                syncToProvider(account, provider, contacts)
                syncResult.stats.numUpdates += contacts.size.toLong()
            } catch (e: Exception) {
                syncResult.stats.numIoExceptions++
            }
        }
    }

    private fun syncToProvider(
        account: Account,
        provider: ContentProviderClient,
        contacts: List<Contact>,
    ) {
        // Clear existing contacts for this account
        provider.delete(
            ContactsContract.RawContacts.CONTENT_URI.buildUpon()
                .appendQueryParameter(ContactsContract.RawContacts.ACCOUNT_NAME, account.name)
                .appendQueryParameter(ContactsContract.RawContacts.ACCOUNT_TYPE, account.type)
                .build(),
            null,
            null,
        )

        // Ensure groups exist and collect group IDs
        val groupIds = mutableMapOf<String, Long>()
        val allGroups = contacts.flatMap { it.groups }.toSet()
        for (groupName in allGroups) {
            val groupId = ensureGroup(account, provider, groupName)
            if (groupId != null) {
                groupIds[groupName] = groupId
            }
        }

        // Insert fresh contacts
        for (contact in contacts) {
            val ops = ArrayList<ContentProviderOperation>()

            // RawContact — star VIPs as favorites
            val isVip = contact.groups.any { it.equals("VIPs", ignoreCase = true) }
            ops.add(
                ContentProviderOperation.newInsert(ContactsContract.RawContacts.CONTENT_URI)
                    .withValue(ContactsContract.RawContacts.ACCOUNT_NAME, account.name)
                    .withValue(ContactsContract.RawContacts.ACCOUNT_TYPE, account.type)
                    .withValue(ContactsContract.RawContacts.SOURCE_ID, contact.id.toString())
                    .withValue(ContactsContract.RawContacts.STARRED, if (isVip) 1 else 0)
                    .build()
            )

            // StructuredName
            ops.add(
                ContentProviderOperation.newInsert(ContactsContract.Data.CONTENT_URI)
                    .withValueBackReference(ContactsContract.Data.RAW_CONTACT_ID, 0)
                    .withValue(ContactsContract.Data.MIMETYPE, ContactsContract.CommonDataKinds.StructuredName.CONTENT_ITEM_TYPE)
                    .withValue(ContactsContract.CommonDataKinds.StructuredName.DISPLAY_NAME, contact.name)
                    .build()
            )

            // Email
            contact.email?.let { email ->
                ops.add(
                    ContentProviderOperation.newInsert(ContactsContract.Data.CONTENT_URI)
                        .withValueBackReference(ContactsContract.Data.RAW_CONTACT_ID, 0)
                        .withValue(ContactsContract.Data.MIMETYPE, ContactsContract.CommonDataKinds.Email.CONTENT_ITEM_TYPE)
                        .withValue(ContactsContract.CommonDataKinds.Email.ADDRESS, email)
                        .withValue(ContactsContract.CommonDataKinds.Email.TYPE, ContactsContract.CommonDataKinds.Email.TYPE_OTHER)
                        .build()
                )
            }

            // Phone
            contact.phone?.let { phone ->
                ops.add(
                    ContentProviderOperation.newInsert(ContactsContract.Data.CONTENT_URI)
                        .withValueBackReference(ContactsContract.Data.RAW_CONTACT_ID, 0)
                        .withValue(ContactsContract.Data.MIMETYPE, ContactsContract.CommonDataKinds.Phone.CONTENT_ITEM_TYPE)
                        .withValue(ContactsContract.CommonDataKinds.Phone.NUMBER, phone)
                        .withValue(ContactsContract.CommonDataKinds.Phone.TYPE, ContactsContract.CommonDataKinds.Phone.TYPE_OTHER)
                        .build()
                )
            }

            // Website
            contact.website?.let { website ->
                ops.add(
                    ContentProviderOperation.newInsert(ContactsContract.Data.CONTENT_URI)
                        .withValueBackReference(ContactsContract.Data.RAW_CONTACT_ID, 0)
                        .withValue(ContactsContract.Data.MIMETYPE, ContactsContract.CommonDataKinds.Website.CONTENT_ITEM_TYPE)
                        .withValue(ContactsContract.CommonDataKinds.Website.URL, website)
                        .build()
                )
            }

            // Group memberships
            for (group in contact.groups) {
                val gid = groupIds[group] ?: continue
                ops.add(
                    ContentProviderOperation.newInsert(ContactsContract.Data.CONTENT_URI)
                        .withValueBackReference(ContactsContract.Data.RAW_CONTACT_ID, 0)
                        .withValue(ContactsContract.Data.MIMETYPE, ContactsContract.CommonDataKinds.GroupMembership.CONTENT_ITEM_TYPE)
                        .withValue(ContactsContract.CommonDataKinds.GroupMembership.GROUP_ROW_ID, gid)
                        .build()
                )
            }

            try {
                context.contentResolver.applyBatch(ContactsContract.AUTHORITY, ops)
            } catch (_: Exception) {
                // Skip individual contact failures
            }
        }
    }

    /**
     * Find or create a contact group for this account, returning its row ID.
     */
    private fun ensureGroup(
        account: Account,
        provider: ContentProviderClient,
        groupName: String,
    ): Long? {
        // Check if the group already exists
        val cursor = provider.query(
            ContactsContract.Groups.CONTENT_URI,
            arrayOf(ContactsContract.Groups._ID),
            "${ContactsContract.Groups.ACCOUNT_NAME} = ? AND ${ContactsContract.Groups.ACCOUNT_TYPE} = ? AND ${ContactsContract.Groups.TITLE} = ?",
            arrayOf(account.name, account.type, groupName),
            null,
        )
        cursor?.use {
            if (it.moveToFirst()) {
                return it.getLong(0)
            }
        }

        // Create the group
        val values = ContentValues().apply {
            put(ContactsContract.Groups.ACCOUNT_NAME, account.name)
            put(ContactsContract.Groups.ACCOUNT_TYPE, account.type)
            put(ContactsContract.Groups.TITLE, groupName)
            put(ContactsContract.Groups.GROUP_VISIBLE, 1)
        }
        val uri = provider.insert(ContactsContract.Groups.CONTENT_URI, values)
        return uri?.lastPathSegment?.toLongOrNull()
    }
}
