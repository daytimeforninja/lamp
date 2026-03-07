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

        // Insert fresh contacts
        for (contact in contacts) {
            val ops = ArrayList<ContentProviderOperation>()

            // RawContact
            ops.add(
                ContentProviderOperation.newInsert(ContactsContract.RawContacts.CONTENT_URI)
                    .withValue(ContactsContract.RawContacts.ACCOUNT_NAME, account.name)
                    .withValue(ContactsContract.RawContacts.ACCOUNT_TYPE, account.type)
                    .withValue(ContactsContract.RawContacts.SOURCE_ID, contact.id.toString())
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

            try {
                context.contentResolver.applyBatch(ContactsContract.AUTHORITY, ops)
            } catch (_: Exception) {
                // Skip individual contact failures
            }
        }
    }
}
