// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

package app.totaldrop.totaldrop

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Pure multi-window drag payload classification tests (#632). */
class DragDropTest {
    @Test
    fun emptyDropProducesEmpty() {
        assertEquals(SharedFiles.DragPayload.Empty, SharedFiles.classifyDrag<String>(emptyList()))
        assertEquals(
            SharedFiles.DragPayload.Empty,
            SharedFiles.classifyDrag(
                listOf(
                    SharedFiles.DragItem<String>(null, null),
                    SharedFiles.DragItem<String>(null, ""),
                ),
            ),
        )
    }

    @Test
    fun fileUrisAreDeduplicatedInOrderAndBeatText() {
        val payload = SharedFiles.classifyDrag(
            listOf(
                SharedFiles.DragItem("content://media/1", "caption"),
                SharedFiles.DragItem<String>(null, "ignored when files exist"),
                SharedFiles.DragItem("content://media/2", null),
                SharedFiles.DragItem("content://media/1", null),
            ),
        )
        assertEquals(
            SharedFiles.DragPayload.Files(listOf("content://media/1", "content://media/2")),
            payload,
        )
    }

    @Test
    fun textBeforeUriStillProducesDeduplicatedFiles() {
        val payload = SharedFiles.classifyDrag(
            listOf(
                SharedFiles.DragItem<String>(null, "leading caption"),
                SharedFiles.DragItem("content://docs/report.pdf", "caption"),
                SharedFiles.DragItem<String>(null, "trailing note"),
                SharedFiles.DragItem("content://docs/report.pdf", null),
                SharedFiles.DragItem("content://docs/slides.pdf", null),
            ),
        )
        assertEquals(
            SharedFiles.DragPayload.Files(
                listOf("content://docs/report.pdf", "content://docs/slides.pdf"),
            ),
            payload,
        )
    }

    @Test
    fun textItemsWithoutUrisAreJoinedSkippingEmptyOnes() {
        val payload = SharedFiles.classifyDrag(
            listOf(
                SharedFiles.DragItem<String>(null, "first line"),
                SharedFiles.DragItem<String>(null, ""),
                SharedFiles.DragItem<String>(null, null),
                SharedFiles.DragItem<String>(null, "second line"),
            ),
        )
        assertEquals(
            SharedFiles.DragPayload.Text("first line\nsecond line"),
            payload,
        )
    }

    @Test
    fun dragStartedAcceptsFilesAndTextAndRejectsInternalOrEmptyDrags() {
        assertTrue(SharedFiles.acceptsDragMimeTypes(listOf("text/plain")))
        assertTrue(SharedFiles.acceptsDragMimeTypes(listOf("text/uri-list")))
        assertTrue(SharedFiles.acceptsDragMimeTypes(listOf("image/jpeg")))
        assertTrue(SharedFiles.acceptsDragMimeTypes(listOf("application/pdf")))
        assertTrue(SharedFiles.acceptsDragMimeTypes(listOf("*/*")))
        assertTrue(
            SharedFiles.acceptsDragMimeTypes(listOf("text/vnd.android.intent", "image/png")),
        )

        assertFalse(SharedFiles.acceptsDragMimeTypes(emptyList()))
        assertFalse(SharedFiles.acceptsDragMimeTypes(listOf("")))
        assertFalse(SharedFiles.acceptsDragMimeTypes(listOf("custom-tag")))
        assertFalse(SharedFiles.acceptsDragMimeTypes(listOf("text/")))
        assertFalse(SharedFiles.acceptsDragMimeTypes(listOf("/plain")))
        assertFalse(
            SharedFiles.acceptsDragMimeTypes(listOf("text/vnd.android.intent")),
        )
        assertFalse(
            SharedFiles.acceptsDragMimeTypes(listOf("application/vnd.android.shortcut")),
        )
    }
}
