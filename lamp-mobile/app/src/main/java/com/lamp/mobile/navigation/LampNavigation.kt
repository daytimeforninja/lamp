package com.lamp.mobile.navigation

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.*
import androidx.compose.ui.graphics.vector.ImageVector

enum class TopLevelDestination(
    val route: String,
    val label: String,
    val selectedIcon: ImageVector,
    val unselectedIcon: ImageVector,
) {
    DAILY_PLANNING(
        route = "daily_planning",
        label = "Plan",
        selectedIcon = Icons.Filled.CalendarMonth,
        unselectedIcon = Icons.Outlined.CalendarMonth,
    ),
    INBOX(
        route = "inbox",
        label = "Inbox",
        selectedIcon = Icons.Filled.Inbox,
        unselectedIcon = Icons.Outlined.Inbox,
    ),
    NEXT_ACTIONS(
        route = "next_actions",
        label = "Next",
        selectedIcon = Icons.Filled.PlayArrow,
        unselectedIcon = Icons.Outlined.PlayArrow,
    ),
    DO_MODE(
        route = "do_mode",
        label = "Do",
        selectedIcon = Icons.Filled.RocketLaunch,
        unselectedIcon = Icons.Outlined.RocketLaunch,
    ),
    MORE(
        route = "more",
        label = "More",
        selectedIcon = Icons.Filled.Menu,
        unselectedIcon = Icons.Outlined.Menu,
    ),
}

enum class DrawerDestination(
    val route: String,
    val label: String,
    val icon: ImageVector,
) {
    ALL_TASKS("all_tasks", "All Tasks", Icons.Outlined.Checklist),
    PROJECTS("projects", "Projects", Icons.Outlined.AccountTree),
    WAITING("waiting", "Waiting For", Icons.Outlined.HourglassBottom),
    SOMEDAY("someday", "Someday/Maybe", Icons.Outlined.LightMode),
    HABITS("habits", "Habits", Icons.Outlined.Loop),
    CALENDAR("calendar", "Calendar", Icons.Outlined.CalendarMonth),
    NOTES("notes", "Notes", Icons.Outlined.Note),
    CONTACTS("contacts", "Contacts", Icons.Outlined.Contacts),
    MEDIA("media", "Media", Icons.Outlined.Movie),
    SHOPPING("shopping", "Shopping", Icons.Outlined.ShoppingCart),
    REVIEW("review", "Review", Icons.Outlined.RateReview),
    SETTINGS("settings", "Settings", Icons.Outlined.Settings),
    CONFLICTS("conflicts", "Conflicts", Icons.Outlined.SyncProblem),
}
