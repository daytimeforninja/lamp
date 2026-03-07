package com.lamp.mobile.navigation

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import com.lamp.mobile.feature.inbox.InboxScreen
import com.lamp.mobile.feature.nextactions.NextActionsScreen
import com.lamp.mobile.feature.projects.ProjectsScreen
import com.lamp.mobile.feature.waiting.WaitingScreen
import com.lamp.mobile.feature.someday.SomedayScreen
import com.lamp.mobile.feature.habits.HabitsScreen
import com.lamp.mobile.feature.dailyplanning.DailyPlanningScreen
import com.lamp.mobile.feature.domode.DoModeScreen
import com.lamp.mobile.feature.calendar.CalendarScreen
import com.lamp.mobile.feature.notes.NotesScreen
import com.lamp.mobile.feature.lists.ListsScreen
import com.lamp.mobile.feature.contacts.ContactsScreen
import com.lamp.mobile.feature.review.ReviewScreen
import com.lamp.mobile.feature.settings.SettingsScreen
import com.lamp.mobile.feature.conflicts.ConflictsScreen

@Composable
fun LampNavHost(
    navController: NavHostController,
    modifier: Modifier = Modifier,
) {
    NavHost(
        navController = navController,
        startDestination = TopLevelDestination.INBOX.route,
        modifier = modifier,
    ) {
        composable(TopLevelDestination.DAILY_PLANNING.route) { DailyPlanningScreen() }
        composable(TopLevelDestination.INBOX.route) { InboxScreen() }
        composable(TopLevelDestination.NEXT_ACTIONS.route) { NextActionsScreen() }
        composable(TopLevelDestination.DO_MODE.route) { DoModeScreen() }

        composable(DrawerDestination.ALL_TASKS.route) { NextActionsScreen() } // Reuse with different filter
        composable(DrawerDestination.PROJECTS.route) { ProjectsScreen() }
        composable(DrawerDestination.WAITING.route) { WaitingScreen() }
        composable(DrawerDestination.SOMEDAY.route) { SomedayScreen() }
        composable(DrawerDestination.HABITS.route) { HabitsScreen() }
        composable(DrawerDestination.CALENDAR.route) { CalendarScreen() }
        composable(DrawerDestination.NOTES.route) { NotesScreen() }
        composable(DrawerDestination.CONTACTS.route) { ContactsScreen() }
        composable(DrawerDestination.MEDIA.route) { ListsScreen(isMedia = true) }
        composable(DrawerDestination.SHOPPING.route) { ListsScreen(isMedia = false) }
        composable(DrawerDestination.REVIEW.route) { ReviewScreen() }
        composable(DrawerDestination.SETTINGS.route) { SettingsScreen() }
        composable(DrawerDestination.CONFLICTS.route) { ConflictsScreen() }
    }
}
