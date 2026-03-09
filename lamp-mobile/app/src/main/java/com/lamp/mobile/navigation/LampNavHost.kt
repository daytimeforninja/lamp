package com.lamp.mobile.navigation

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.navArgument
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
import com.lamp.mobile.feature.taskdetail.TaskDetailScreen
import com.lamp.mobile.feature.alltasks.AllTasksScreen
import com.lamp.mobile.feature.agenda.AgendaScreen
import com.lamp.mobile.feature.accounts.AccountsScreen

@Composable
fun LampNavHost(
    navController: NavHostController,
    modifier: Modifier = Modifier,
) {
    val navigateToTaskDetail: (java.util.UUID) -> Unit = { taskId ->
        navController.navigate("task_detail/$taskId")
    }

    NavHost(
        navController = navController,
        startDestination = TopLevelDestination.INBOX.route,
        modifier = modifier,
    ) {
        composable(TopLevelDestination.DAILY_PLANNING.route) {
            DailyPlanningScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(TopLevelDestination.INBOX.route) {
            InboxScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(TopLevelDestination.NEXT_ACTIONS.route) {
            NextActionsScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(TopLevelDestination.DO_MODE.route) {
            DoModeScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }

        composable(DrawerDestination.ALL_TASKS.route) {
            AllTasksScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(DrawerDestination.PROJECTS.route) {
            ProjectsScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(DrawerDestination.WAITING.route) {
            WaitingScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(DrawerDestination.SOMEDAY.route) {
            SomedayScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(DrawerDestination.HABITS.route) { HabitsScreen() }
        composable(DrawerDestination.CALENDAR.route) { CalendarScreen() }
        composable(DrawerDestination.NOTES.route) { NotesScreen() }
        composable(DrawerDestination.CONTACTS.route) { ContactsScreen() }
        composable(DrawerDestination.MEDIA.route) { ListsScreen(isMedia = true) }
        composable(DrawerDestination.SHOPPING.route) { ListsScreen(isMedia = false) }
        composable(DrawerDestination.REVIEW.route) { ReviewScreen() }
        composable(DrawerDestination.SETTINGS.route) { SettingsScreen() }
        composable(DrawerDestination.CONFLICTS.route) { ConflictsScreen() }
        composable(DrawerDestination.AGENDA.route) {
            AgendaScreen(onNavigateToTaskDetail = navigateToTaskDetail)
        }
        composable(DrawerDestination.ACCOUNTS.route) { AccountsScreen() }

        // Task detail route
        composable(
            route = "task_detail/{taskId}",
            arguments = listOf(navArgument("taskId") { type = NavType.StringType }),
        ) {
            TaskDetailScreen(onBack = { navController.popBackStack() })
        }
    }
}
