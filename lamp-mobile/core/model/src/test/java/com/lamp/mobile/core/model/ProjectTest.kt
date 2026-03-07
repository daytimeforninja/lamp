package com.lamp.mobile.core.model

import org.junit.Assert.*
import org.junit.Test

class ProjectTest {

    @Test
    fun `nextAction returns first NEXT task`() {
        val tasks = listOf(
            Task(title = "A", state = TaskState.TODO),
            Task(title = "B", state = TaskState.NEXT),
            Task(title = "C", state = TaskState.TODO),
        )
        val project = Project(name = "P", tasks = tasks)
        assertEquals("B", project.nextAction()?.title)
    }

    @Test
    fun `nextAction falls back to first TODO`() {
        val tasks = listOf(
            Task(title = "A", state = TaskState.WAITING),
            Task(title = "B", state = TaskState.TODO),
        )
        val project = Project(name = "P", tasks = tasks)
        assertEquals("B", project.nextAction()?.title)
    }

    @Test
    fun `isStuck when active tasks but no next action`() {
        val tasks = listOf(
            Task(title = "A", state = TaskState.WAITING),
            Task(title = "B", state = TaskState.SOMEDAY),
        )
        val project = Project(name = "P", tasks = tasks)
        assertTrue(project.isStuck())
    }

    @Test
    fun `not stuck when has TODO task`() {
        val tasks = listOf(
            Task(title = "A", state = TaskState.TODO),
        )
        val project = Project(name = "P", tasks = tasks)
        assertFalse(project.isStuck())
    }

    @Test
    fun `completionRatio counts done tasks`() {
        val tasks = listOf(
            Task(title = "A", state = TaskState.DONE),
            Task(title = "B", state = TaskState.CANCELLED),
            Task(title = "C", state = TaskState.TODO),
        )
        val project = Project(name = "P", tasks = tasks)
        assertEquals(2 to 3, project.completionRatio())
    }
}
