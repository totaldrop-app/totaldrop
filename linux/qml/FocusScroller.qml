import QtQuick
import app.totaldrop

// Scrolls `view` so that a control inside `area` that takes keyboard
// focus (Tab) is in view, with a margin. Focus from a click does not
// scroll: the control would move away under the pointer.
QtObject {
    id: scroller

    required property Flickable view
    required property Item area

    property Connections watch: Connections {
        // The page may go before this object.
        target: scroller.area ? scroller.area.Window.window : null
        function onActiveFocusItemChanged() {
            if (scroller.area) scroller.reveal(scroller.area.Window.activeFocusItem)
        }
    }

    function reveal(item) {
        let inside = false
        for (let p = item; p && !inside; p = p.parent) inside = p === area
        const keyboard = item && (item.focusReason === Qt.TabFocusReason
                                  || item.focusReason === Qt.BacktabFocusReason)
        if (!inside || !keyboard) return
        const top = item.mapToItem(view.contentItem, 0, 0).y
        const bottom = top + item.height
        if (top < view.contentY + Theme.s24)
            view.contentY = Math.max(0, top - Theme.s24)
        else if (bottom > view.contentY + view.height - Theme.s24)
            view.contentY = Math.max(0, Math.min(view.contentHeight - view.height,
                                                 bottom - view.height + Theme.s24))
    }
}
