package dev.winportkill.jetbrains.ui

import com.intellij.icons.AllIcons
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBPanel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.components.JBTextArea
import com.intellij.ui.components.JBTextField
import com.intellij.ui.table.JBTable
import dev.winportkill.jetbrains.model.PortEntry
import dev.winportkill.jetbrains.model.PortResponse
import dev.winportkill.jetbrains.model.ProcessEntry
import dev.winportkill.jetbrains.model.ProcessResponse
import dev.winportkill.jetbrains.model.ViewMode
import dev.winportkill.jetbrains.sidecar.SidecarManager
import java.awt.BorderLayout
import java.awt.CardLayout
import java.awt.Component
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.Insets
import java.awt.event.ComponentAdapter
import java.awt.event.ComponentEvent
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import java.text.DecimalFormat
import javax.swing.AbstractCellEditor
import javax.swing.BorderFactory
import javax.swing.ButtonGroup
import javax.swing.JButton
import javax.swing.JMenuItem
import javax.swing.JPanel
import javax.swing.JPopupMenu
import javax.swing.JRadioButton
import javax.swing.JTable
import javax.swing.SwingUtilities
import javax.swing.Timer
import javax.swing.event.DocumentEvent
import javax.swing.event.DocumentListener
import javax.swing.table.DefaultTableModel
import javax.swing.table.TableCellEditor
import javax.swing.table.TableCellRenderer

private const val COMPACT_THRESHOLD = 500
private const val MEDIUM_THRESHOLD = 860

private enum class LayoutBand {
    WIDE,
    MEDIUM,
    COMPACT,
}

class WinPortKillPanel(
    private val project: Project,
) : JPanel(BorderLayout()) {
    private val filterField = JBTextField()
    private val detailsLabel = JBLabel("Select a row to see details.")
    private val detailArea = JBTextArea()
    private val statusLabel = JBLabel("Starting WinPortKill...")

    private val portsModel = nonEditableTableModel()
    private val processesModel = nonEditableTableModel()
    private val portsTable = JBTable(portsModel)
    private val processesTable = JBTable(processesModel)

    private val toolbarPanel = JPanel(GridBagLayout())
    private val footerPanel = JBPanel<JBPanel<*>>(BorderLayout(0, 4))
    private val contentLayout = CardLayout()
    private val contentPanel = JPanel(contentLayout)

    private val portsButton = JRadioButton("Ports", true)
    private val processesButton = JRadioButton("Processes")
    private val refreshButton = JButton("Refresh", AllIcons.Actions.Refresh)
    private val killButton = JButton("Kill Selected")
    private val filterTimer = Timer(220) { refresh() }

    private val memoryFormat = DecimalFormat("0.0")

    private var mode = ViewMode.PORTS
    private var latestPorts: List<PortEntry> = emptyList()
    private var latestProcesses: List<ProcessEntry> = emptyList()
    private var currentBand = LayoutBand.WIDE

    init {
        ButtonGroup().apply {
            add(portsButton)
            add(processesButton)
        }

        filterTimer.isRepeats = false
        filterField.emptyText.text = "Filter by pid, process, port, address..."
        filterField.toolTipText = "Filter by pid, process, port, address, or protocol"

        configureTable(portsTable)
        configureTable(processesTable)
        installInteractions(portsTable)
        installInteractions(processesTable)

        footerPanel.add(detailsLabel, BorderLayout.NORTH)
        footerPanel.add(
            JBScrollPane(detailArea).apply {
                border = BorderFactory.createEmptyBorder()
                horizontalScrollBarPolicy = JBScrollPane.HORIZONTAL_SCROLLBAR_NEVER
                verticalScrollBarPolicy = JBScrollPane.VERTICAL_SCROLLBAR_AS_NEEDED
            },
            BorderLayout.CENTER
        )
        footerPanel.add(statusLabel, BorderLayout.SOUTH)
        footerPanel.border = BorderFactory.createEmptyBorder(6, 0, 0, 0)

        detailArea.isEditable = false
        detailArea.lineWrap = true
        detailArea.wrapStyleWord = true
        detailArea.rows = 3
        detailArea.border = BorderFactory.createEmptyBorder(2, 0, 2, 0)

        contentPanel.add(JBScrollPane(portsTable), ViewMode.PORTS.name)
        contentPanel.add(JBScrollPane(processesTable), ViewMode.PROCESSES.name)

        add(toolbarPanel, BorderLayout.NORTH)
        add(contentPanel, BorderLayout.CENTER)
        add(footerPanel, BorderLayout.SOUTH)

        portsButton.addActionListener {
            mode = ViewMode.PORTS
            switchMode()
            refresh()
        }
        processesButton.addActionListener {
            mode = ViewMode.PROCESSES
            switchMode()
            refresh()
        }
        refreshButton.addActionListener { refresh() }
        killButton.addActionListener { killSelected() }
        filterField.addActionListener { refresh() }
        filterField.document.addDocumentListener(object : DocumentListener {
            override fun insertUpdate(e: DocumentEvent) = scheduleRefresh()
            override fun removeUpdate(e: DocumentEvent) = scheduleRefresh()
            override fun changedUpdate(e: DocumentEvent) = scheduleRefresh()
        })

        addComponentListener(object : ComponentAdapter() {
            override fun componentResized(e: ComponentEvent) {
                val nextBand = resolveBand()
                if (nextBand != currentBand) {
                    currentBand = nextBand
                    rebuildVisibleTable()
                    relayoutToolbar()
                    relayoutFooter()
                } else {
                    relayoutToolbar()
                }
            }
        })

        currentBand = resolveBand()
        switchMode()
        relayoutToolbar()
        relayoutFooter()
        refresh()
    }

    fun refresh() {
        statusLabel.text = "Loading WinPortKill..."
        val filter = filterField.text.trim()

        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                val apiClient = SidecarManager.getInstance(project).ensureStarted()
                when (mode) {
                    ViewMode.PORTS -> {
                        val payload = apiClient.fetchPorts(filter)
                        latestPorts = payload.entries
                        SwingUtilities.invokeLater { renderPorts(payload) }
                    }

                    ViewMode.PROCESSES -> {
                        val payload = apiClient.fetchProcesses(filter)
                        latestProcesses = payload.entries
                        SwingUtilities.invokeLater { renderProcesses(payload) }
                    }
                }
            } catch (t: Throwable) {
                SwingUtilities.invokeLater {
                    statusLabel.text = t.message ?: t.javaClass.simpleName
                }
            }
        }
    }

    private fun renderPorts(payload: PortResponse) {
        latestPorts = payload.entries
        rebuildPortsTable()
        updateDetails()
        statusLabel.text =
            "Rows ${payload.stats.totalRows}  Proc ${payload.stats.totalProcs}  TCP ${payload.stats.tcpCount}  UDP ${payload.stats.udpCount}"
    }

    private fun renderProcesses(payload: ProcessResponse) {
        latestProcesses = payload.entries
        rebuildProcessesTable()
        updateDetails()
        statusLabel.text =
            "Proc ${payload.stats.totalProcs}  Active ${payload.stats.procsWithPorts}  TCP ${payload.stats.tcpCount}  Bindings ${payload.stats.totalPortBindings}"
    }

    private fun rebuildVisibleTable() {
        when (mode) {
            ViewMode.PORTS -> rebuildPortsTable()
            ViewMode.PROCESSES -> rebuildProcessesTable()
        }
        updateDetails()
    }

    private fun rebuildPortsTable() {
        portsModel.setRowCount(0)
        when (currentBand) {
            LayoutBand.WIDE -> {
                portsModel.setColumnIdentifiers(arrayOf("Proto", "Addr", "Port", "PID", "Mem(MB)", "Process", ""))
                latestPorts.forEach { entry ->
                    portsModel.addRow(
                        arrayOf<Any>(
                            entry.proto,
                            entry.localAddr,
                            entry.port,
                            entry.pid,
                            memoryMb(entry.memory),
                            entry.name,
                            "Kill",
                        )
                    )
                }
                setWidths(portsTable, intArrayOf(56, 96, 64, 72, 78, 200, 72))
            }

            LayoutBand.MEDIUM -> {
                portsModel.setColumnIdentifiers(arrayOf("Proto", "Endpoint", "PID", "Process", ""))
                latestPorts.forEach { entry ->
                    portsModel.addRow(
                        arrayOf<Any>(
                            entry.proto,
                            "${entry.localAddr}:${entry.port}",
                            entry.pid,
                            entry.name,
                            "Kill",
                        )
                    )
                }
                setWidths(portsTable, intArrayOf(56, 170, 72, 150, 72))
            }

            LayoutBand.COMPACT -> {
                portsModel.setColumnIdentifiers(arrayOf("Endpoint", "Process", ""))
                latestPorts.forEach { entry ->
                    portsModel.addRow(
                        arrayOf<Any>(
                            "${entry.proto} ${entry.localAddr}:${entry.port}",
                            "${entry.name}",
                            "Kill",
                        )
                    )
                }
                setWidths(portsTable, intArrayOf(210, 120, 72))
            }
        }
        wireKillColumn(portsTable) { row -> latestPorts.getOrNull(row)?.pid }
    }

    private fun rebuildProcessesTable() {
        processesModel.setRowCount(0)
        when (currentBand) {
            LayoutBand.WIDE -> {
                processesModel.setColumnIdentifiers(arrayOf("PID", "TCP", "UDP", "Mem(MB)", "Process", "Ports", ""))
                latestProcesses.forEach { entry ->
                    processesModel.addRow(
                        arrayOf<Any>(
                            entry.pid,
                            entry.tcpPorts,
                            entry.udpPorts,
                            memoryMb(entry.memory),
                            entry.name,
                            entry.ports.take(3).joinToString(" | ") { "${it.proto} ${it.localAddr}:${it.port}" }
                                .ifBlank { "No listening ports" },
                            "Kill",
                        )
                    )
                }
                setWidths(processesTable, intArrayOf(72, 44, 44, 78, 170, 260, 72))
            }

            LayoutBand.MEDIUM -> {
                processesModel.setColumnIdentifiers(arrayOf("PID", "TCP", "UDP", "Mem(MB)", "Process", ""))
                latestProcesses.forEach { entry ->
                    processesModel.addRow(
                        arrayOf<Any>(
                            entry.pid,
                            entry.tcpPorts,
                            entry.udpPorts,
                            memoryMb(entry.memory),
                            entry.name,
                            "Kill",
                        )
                    )
                }
                setWidths(processesTable, intArrayOf(72, 44, 44, 78, 170, 72))
            }

            LayoutBand.COMPACT -> {
                processesModel.setColumnIdentifiers(arrayOf("Process", "PID", "Bindings", ""))
                latestProcesses.forEach { entry ->
                    processesModel.addRow(
                        arrayOf<Any>(
                            entry.name,
                            entry.pid,
                            "${entry.ports.size} bindings",
                            "Kill",
                        )
                    )
                }
                setWidths(processesTable, intArrayOf(140, 68, 96, 72))
            }
        }
        wireKillColumn(processesTable) { row -> latestProcesses.getOrNull(row)?.pid }
    }

    private fun relayoutToolbar() {
        toolbarPanel.removeAll()
        toolbarPanel.border = BorderFactory.createEmptyBorder(0, 0, 6, 0)

        val modePanel = JPanel().apply {
            isOpaque = false
            add(portsButton)
            add(processesButton)
        }
        val actionsPanel = JPanel().apply {
            isOpaque = false
            add(refreshButton)
            if (currentBand == LayoutBand.WIDE) {
                add(killButton)
            }
        }

        if (currentBand == LayoutBand.COMPACT) {
            toolbarPanel.add(
                modePanel,
                gbc(0, 0, weightx = 1.0, fill = GridBagConstraints.HORIZONTAL, insets = Insets(0, 0, 0, 6))
            )
            toolbarPanel.add(actionsPanel, gbc(1, 0, anchor = GridBagConstraints.EAST))
            toolbarPanel.add(
                filterField,
                gbc(0, 1, gridwidth = 2, weightx = 1.0, fill = GridBagConstraints.HORIZONTAL, insets = Insets(6, 0, 0, 0))
            )
        } else {
            toolbarPanel.add(modePanel, gbc(0, 0))
            toolbarPanel.add(
                filterField,
                gbc(1, 0, weightx = 1.0, fill = GridBagConstraints.HORIZONTAL, insets = Insets(0, 6, 0, 6))
            )
            toolbarPanel.add(actionsPanel, gbc(2, 0, anchor = GridBagConstraints.EAST))
        }

        toolbarPanel.revalidate()
        toolbarPanel.repaint()
    }

    private fun relayoutFooter() {
        detailsLabel.isVisible = true
        detailArea.isVisible = true
        footerPanel.revalidate()
        footerPanel.repaint()
    }

    private fun switchMode() {
        contentLayout.show(contentPanel, mode.name)
    }

    private fun updateDetails() {
        val state = when (mode) {
            ViewMode.PORTS -> {
                val entry = selectedPortEntry()
                if (entry == null) {
                    DetailState(
                        summary = "Select a port row to see details.",
                        body = "",
                    )
                } else {
                    DetailState(
                        summary = "${entry.name} | PID ${entry.pid} | ${memoryMb(entry.memory)} MB",
                        body = "${entry.proto} ${entry.localAddr}:${entry.port}",
                    )
                }
            }

            ViewMode.PROCESSES -> {
                val entry = selectedProcessEntry()
                if (entry == null) {
                    DetailState(
                        summary = "Select a process row to see details.",
                        body = "",
                    )
                } else {
                    val ports = entry.ports.joinToString("\n") { "${it.proto} ${it.localAddr}:${it.port}" }
                        .ifBlank { "No listening ports" }
                    if (currentBand == LayoutBand.COMPACT) {
                        DetailState(
                            summary = "${entry.name} | PID ${entry.pid} | ${entry.ports.size} bindings",
                            body = ports,
                        )
                    } else {
                        DetailState(
                            summary = "${entry.name} | PID ${entry.pid} | TCP ${entry.tcpPorts} UDP ${entry.udpPorts} | ${memoryMb(entry.memory)} MB",
                            body = ports,
                        )
                    }
                }
            }
        }
        detailsLabel.text = state.summary
        detailArea.text = state.body
        detailArea.caretPosition = 0
    }

    private fun scheduleRefresh() {
        filterTimer.restart()
    }

    private fun resolveBand(): LayoutBand {
        return when {
            width in 1..COMPACT_THRESHOLD -> LayoutBand.COMPACT
            width <= MEDIUM_THRESHOLD -> LayoutBand.MEDIUM
            else -> LayoutBand.WIDE
        }
    }

    private fun configureTable(table: JTable) {
        table.autoCreateRowSorter = true
        table.fillsViewportHeight = true
        table.autoResizeMode = JTable.AUTO_RESIZE_ALL_COLUMNS
        table.rowHeight = 24
    }

    private fun installInteractions(table: JTable) {
        table.selectionModel.addListSelectionListener { updateDetails() }
        table.addMouseListener(object : MouseAdapter() {
            override fun mousePressed(event: MouseEvent) = maybeShowContextMenu(table, event)
            override fun mouseReleased(event: MouseEvent) = maybeShowContextMenu(table, event)
        })
    }

    private fun maybeShowContextMenu(table: JTable, event: MouseEvent) {
        if (!event.isPopupTrigger) {
            return
        }
        val row = table.rowAtPoint(event.point)
        if (row >= 0) {
            table.setRowSelectionInterval(row, row)
        }
        val menu = JPopupMenu()
        menu.add(JMenuItem("Kill PID").apply {
            addActionListener { killSelected() }
        })
        menu.show(table, event.x, event.y)
    }

    private fun wireKillColumn(table: JTable, pidProvider: (modelRow: Int) -> Int?) {
        val killColumnIndex = table.columnModel.columnCount - 1
        table.columnModel.getColumn(killColumnIndex).cellRenderer = KillButtonRenderer()
        table.columnModel.getColumn(killColumnIndex).cellEditor = KillButtonEditor(table, pidProvider)
    }

    private fun setWidths(table: JTable, widths: IntArray) {
        repeat(table.columnModel.columnCount) { index ->
            val column = table.columnModel.getColumn(index)
            column.preferredWidth = widths[index]
            if (index == widths.lastIndex) {
                column.minWidth = 76
                column.maxWidth = 76
                column.width = 76
            } else {
                column.minWidth = when (currentBand) {
                    LayoutBand.WIDE -> 44
                    LayoutBand.MEDIUM -> 40
                    LayoutBand.COMPACT -> 36
                }
                column.maxWidth = Int.MAX_VALUE
            }
        }
    }

    private fun killSelected() {
        val pid = when (mode) {
            ViewMode.PORTS -> selectedPortEntry()?.pid
            ViewMode.PROCESSES -> selectedProcessEntry()?.pid
        } ?: run {
            statusLabel.text = "Select a row first."
            return
        }
        killByPid(pid)
    }

    private fun selectedPortEntry(): PortEntry? {
        val row = portsTable.selectedRow
        if (row < 0) {
            return null
        }
        return latestPorts.getOrNull(portsTable.convertRowIndexToModel(row))
    }

    private fun selectedProcessEntry(): ProcessEntry? {
        val row = processesTable.selectedRow
        if (row < 0) {
            return null
        }
        return latestProcesses.getOrNull(processesTable.convertRowIndexToModel(row))
    }

    private fun killByPid(pid: Int) {
        val confirmed = Messages.showYesNoDialog(
            project,
            "Terminate PID $pid?",
            "WinPortKill",
            Messages.getQuestionIcon()
        ) == Messages.YES

        if (!confirmed) {
            statusLabel.text = "Kill cancelled"
            return
        }

        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                val apiClient = SidecarManager.getInstance(project).ensureStarted()
                val result = apiClient.kill(pid)
                SwingUtilities.invokeLater {
                    statusLabel.text = result.message
                    refresh()
                }
            } catch (t: Throwable) {
                SwingUtilities.invokeLater {
                    statusLabel.text = "Kill failed: ${t.message ?: t.javaClass.simpleName}"
                }
            }
        }
    }

    private fun gbc(
        gridx: Int,
        gridy: Int,
        gridwidth: Int = 1,
        weightx: Double = 0.0,
        fill: Int = GridBagConstraints.NONE,
        anchor: Int = GridBagConstraints.WEST,
        insets: Insets = Insets(0, 0, 0, 0),
    ): GridBagConstraints {
        return GridBagConstraints().apply {
            this.gridx = gridx
            this.gridy = gridy
            this.gridwidth = gridwidth
            this.weightx = weightx
            this.fill = fill
            this.anchor = anchor
            this.insets = insets
        }
    }

    private fun nonEditableTableModel(): DefaultTableModel {
        return object : DefaultTableModel() {
            override fun isCellEditable(row: Int, column: Int): Boolean = column == columnCount - 1
        }
    }

    private fun memoryMb(bytes: Long): String {
        return memoryFormat.format(bytes / 1024.0 / 1024.0)
    }

    private data class DetailState(
        val summary: String,
        val body: String,
    )

    private inner class KillButtonRenderer : JButton("Kill"), TableCellRenderer {
        override fun getTableCellRendererComponent(
            table: JTable,
            value: Any?,
            isSelected: Boolean,
            hasFocus: Boolean,
            row: Int,
            column: Int,
        ): Component {
            text = "Kill"
            return this
        }
    }

    private inner class KillButtonEditor(
        private val table: JTable,
        private val pidProvider: (modelRow: Int) -> Int?,
    ) : AbstractCellEditor(), TableCellEditor {
        private val button = JButton("Kill")

        init {
            button.addActionListener {
                val editingRow = table.editingRow
                if (editingRow >= 0) {
                    table.setRowSelectionInterval(editingRow, editingRow)
                    val modelRow = table.convertRowIndexToModel(editingRow)
                    val pid = pidProvider(modelRow)
                    stopCellEditing()
                    if (pid != null) {
                        killByPid(pid)
                    }
                } else {
                    stopCellEditing()
                }
            }
        }

        override fun getCellEditorValue(): Any = "Kill"

        override fun getTableCellEditorComponent(
            table: JTable,
            value: Any?,
            isSelected: Boolean,
            row: Int,
            column: Int,
        ): Component {
            return button
        }
    }
}
