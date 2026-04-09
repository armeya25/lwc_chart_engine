/**
 * Command Dispatcher & Handlers
 */
import { SyncManager } from './core.js';
import { showNotification, createLayout, hideLoader } from './ui.js';

export const CommandQueue = {
    queue: [],
    isProcessing: false,
    BUDGET_MS: 8,

    push: function (cmd) {
        this.queue.push(cmd);
        if (!this.isProcessing) {
            this.isProcessing = true;
            requestAnimationFrame(() => this.process());
        }
    },

    process: function () {
        const start = performance.now();
        while (this.queue.length > 0) {
            if (performance.now() - start > this.BUDGET_MS) {
                requestAnimationFrame(() => this.process()); return;
            }
            
            // Peak at the next command instead of shifting immediately
            const cmd = this.queue[0];
            const wasProcessed = this.processCommandSync(cmd);
            
            if (wasProcessed) {
                this.queue.shift(); // Only remove if successfully processed
            } else {
                // If not ready, wait for the next frame
                requestAnimationFrame(() => this.process());
                return;
            }
        }
        this.isProcessing = false;
    },

    processCommandSync: function (cmd) {
        const { action, chartId = 'chart-0' } = cmd;
        const handler = CommandHandlers[action];
        
        if (!handler) {
            console.warn("Unknown command action:", action);
            return true; // Mark as processed to remove from queue
        }

        try {
            const targetChart = window.charts.get(chartId);
            
            // Core safety: If the target chart is missing and it's not a global command, 
            // signal that we aren't ready to process this yet.
            const isGlobal = (action === 'set_layout' || action === 'show_notification' || action === 'hide_loading');
            if (!targetChart && !isGlobal) {
                return false; 
            }

            handler(targetChart, cmd, chartId);
            return true;
        } catch (e) {
            console.error(`Error executing ${action} on ${chartId}:`, e);
            const status = document.querySelector('.loading-text');
            if (status) status.innerText = `Error: ${action}`;
        }
    }
};

const getSId = (cmd) => cmd.id || cmd.seriesId || cmd.series_id;

window.isReady = false;
export function handleCommand(cmd) {
    if (typeof cmd === 'string') cmd = JSON.parse(cmd);
    CommandQueue.push(cmd);
};

export const CommandHandlers = {
    configure_chart: (targetChart, cmd) => { if (targetChart) targetChart.applyOptions(cmd.data); },
    set_layout: (_targetChart, cmd) => { 
        const type = cmd.layout || cmd.data?.type || 'single';
        createLayout(type); 
        if (hideLoader) hideLoader();
    },
    create_line_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;
        const series = targetChart.addSeries(LightweightCharts.LineSeries, cmd.options);
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        window.addLegendItem(sid, cmd.name, cmd.options.color);
    },
    create_area_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;
        const series = targetChart.addSeries(LightweightCharts.AreaSeries, cmd.options);
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        window.addLegendItem(sid, cmd.name, cmd.options.lineColor || cmd.options.topColor);
    },
    create_band_plugin: (_targetChart, cmd) => {
        const ownerSeries = window.seriesMap.get(getSId(cmd));
        if (ownerSeries && window.BandSeriesPrimitive) {
            if (!ownerSeries._bandPlugins) ownerSeries._bandPlugins = {};
            if (ownerSeries._bandPlugins[cmd.color]) ownerSeries._bandPlugins[cmd.color].setData(cmd.data);
            else {
                const band = new window.BandSeriesPrimitive({ color: cmd.color });
                ownerSeries.attachPrimitive(band);
                ownerSeries._bandPlugins[cmd.color] = band;
                band.setData(cmd.data);
            }
        }
    },
    create_candlestick_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;
        const series = targetChart.addSeries(LightweightCharts.CandlestickSeries, cmd.options);
        series._name = cmd.name;
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        window.addLegendItem(sid, cmd.name, cmd.options.upColor);
    },
    create_histogram_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;
        const series = targetChart.addSeries(LightweightCharts.HistogramSeries, cmd.options);
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        window.addLegendItem(sid, cmd.name, cmd.options.color);
    },
    set_data: (_targetChart, cmd) => {
        const sid = getSId(cmd);
        const series = window.seriesMap.get(sid);
        if (series) {
            // Auto-fallback: if it's a line/area series and data has no 'value', use 'close'
            const processedData = cmd.data.map(item => {
                if (item.value === undefined && item.close !== undefined) {
                    return { ...item, value: item.close };
                }
                return item;
            });
            series.setData(processedData);
            if (_targetChart && cmd.data.length > 5) _targetChart.timeScale().fitContent();
            if (typeof BoxManager !== 'undefined') BoxManager.updatePositions();
            if (typeof PositionToolManager !== 'undefined') PositionToolManager.updatePositions();
            if (typeof LineToolManager !== 'undefined') LineToolManager.updatePositions();
        }
    },
    update_data: (_targetChart, cmd) => {
        const sid = getSId(cmd);
        const series = window.seriesMap.get(sid);
        if (series) {
            try {
                series.update(cmd.data);
            } catch (e) {
                if (!e.message.includes('oldest data')) {
                    console.error(`Update failed for ${cmd.id}:`, e.message);
                    throw e;
                }
            }
        }
    },
    update_series_options: (_targetChart, cmd) => {
        const series = window.seriesMap.get(getSId(cmd));
        if (series) series.applyOptions(cmd.options);
    },
    remove_series: (targetChart, cmd) => {
        const sid = getSId(cmd);
        const series = window.seriesMap.get(sid);
        if (series && targetChart) {
            targetChart.removeSeries(series);
            window.seriesMap.delete(sid);
            // Also clean up from chartSeriesMap
            window.charts.forEach((c, chartId) => {
                const sSet = window.chartSeriesMap.get(chartId);
                if (sSet) sSet.delete(sid);
            });
            // Remove legend item
            const item = document.getElementById(`legend-item-${sid}`);
            if (item) item.remove();
        }
    },
    clear_all_series: (targetChart, _cmd, chartId) => {
        if (!targetChart) return;
        const sSet = window.chartSeriesMap.get(chartId);
        if (sSet) {
            sSet.forEach(sid => {
                const series = window.seriesMap.get(sid);
                if (series) targetChart.removeSeries(series);
                window.seriesMap.delete(sid);
                const item = document.getElementById(`legend-item-${sid}`);
                if (item) item.remove();
            });
            sSet.clear();
        }
    },
    fit_content: (targetChart) => { if (targetChart) targetChart.timeScale().fitContent(); },
    set_visible_range: (targetChart, cmd) => { if (targetChart) targetChart.timeScale().setVisibleRange(cmd.data); },
    create_position: (_targetChart, cmd, chartId) => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.create(chartId, cmd.id, cmd.data); },
    update_position: (_targetChart, cmd) => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.update(cmd.id, cmd.data); },
    remove_position: (_targetChart, cmd) => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.remove(cmd.id); },
    create_box: (_targetChart, cmd, chartId) => { 
        if (typeof BoxManager !== 'undefined') {
            BoxManager.createBox(chartId, cmd.id, cmd.data); 
        }
    },
    update_box: (_targetChart, cmd) => { if (typeof BoxManager !== 'undefined') BoxManager.updateBox(cmd.id, cmd.data); },
    remove_box: (_targetChart, cmd) => { if (typeof BoxManager !== 'undefined') BoxManager.removeBox(cmd.id); },
    create_line_tool: (_targetChart, cmd, chartId) => { if (typeof LineToolManager !== 'undefined') LineToolManager.create(chartId, cmd.id, cmd.data); },
    update_line_tool: (_targetChart, cmd) => { if (typeof LineToolManager !== 'undefined') LineToolManager.update(cmd.id, cmd.data); },
    remove_line_tool: (_targetChart, cmd) => { if (typeof LineToolManager !== 'undefined') LineToolManager.remove(cmd.id); },
    add_marker: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.addMarker(getSId(cmd), cmd.data, chartId); },
    add_markers_bulk: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.addMarkersBulk(getSId(cmd), cmd.data, chartId); },
    remove_marker: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.removeMarker(getSId(cmd), cmd.marker_id, chartId); },
    update_marker: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.updateMarker(getSId(cmd), cmd.marker_id, cmd.data, chartId); },
    remove_all_markers: () => { if (typeof MarkerManager !== 'undefined') MarkerManager.clearAll(); },
    remove_all_positions: () => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.removeAll(); },
    remove_all_boxes: () => { if (typeof BoxManager !== 'undefined') BoxManager.removeAll(); },
    remove_all_line_tools: () => { if (typeof LineToolManager !== 'undefined') LineToolManager.removeAll(); },
    create_price_line: (_targetChart, cmd) => { if (typeof PriceLineManager !== 'undefined') PriceLineManager.create(getSId(cmd), cmd.line_id, cmd.options); },
    remove_price_line: (_targetChart, cmd) => { if (typeof PriceLineManager !== 'undefined') PriceLineManager.remove(cmd.line_id); },
    update_price_line: (_targetChart, cmd) => { if (typeof PriceLineManager !== 'undefined') PriceLineManager.update(cmd.line_id, cmd.options); },
    set_watermark: (targetChart, cmd) => {
        if (!targetChart) return;
        const d = cmd.data || {};
        if (typeof LightweightCharts.createTextWatermark === 'function') {
            const pane = targetChart.panes()[0];
            if (pane) LightweightCharts.createTextWatermark(pane, { 
                horzAlign: d.horzAlign || 'center', 
                vertAlign: d.vertAlign || 'center', 
                lines: [{ text: d.text, color: d.color || 'rgba(255, 255, 255, 0.1)', fontSize: d.fontSize || 48, fontWeight: 'bold' }] 
            });
        } else {
            targetChart.applyOptions({ 
                watermark: { 
                    visible: true, 
                    text: d.text, 
                    color: d.color || 'rgba(255, 255, 255, 0.1)', 
                    horzAlign: d.horzAlign || 'center', 
                    vertAlign: d.vertAlign || 'center', 
                    fontSize: d.fontSize || 48 
                } 
            });
        }
    },
    set_tooltip: (_targetChart, cmd) => {
        window.tooltipEnabled = !!cmd.data.enabled;
        if (!window.tooltipEnabled) document.querySelectorAll('.floating-tooltip').forEach(el => el.style.opacity = '0');
    },
    configure_price_scale: (targetChart, cmd) => {
        if (!targetChart) return;
        const d = cmd.data, scaleId = d.scaleId || 'right';
        targetChart.priceScale(scaleId).applyOptions({ visible: true, mode: d.mode !== undefined ? d.mode : 0, autoScale: d.autoScale !== undefined ? d.autoScale : true, invertScale: d.invertScale || false, scaleMargins: d.scaleMargins || { top: 0.1, bottom: 0.1 } });
    },
    set_sync: (_targetChart, cmd) => {
        SyncManager.enabled = !!cmd.data.enabled;
        if (!SyncManager.enabled) {
            if (SyncManager._rafId) { cancelAnimationFrame(SyncManager._rafId); SyncManager._rafId = null; }
            window.charts.forEach(c => c.clearCrosshairPosition());
        } else {
            window.charts.forEach((c, id) => SyncManager.register(c, document.getElementById(id.replace('chart-', 'chart-cell-'))));
        }
    },
    set_crosshair_mode: (targetChart, cmd) => { if (targetChart) targetChart.applyOptions({ crosshair: { mode: cmd.data.mode } }); },
    set_timezone: (_targetChart, cmd) => {
        window.chartTimeZone = cmd.data.timezone || 'UTC';
        const localization = { timeFormatter: (ts) => typeof ts !== 'number' ? String(ts) : new Date(ts * 1000).toLocaleString('en-GB', { timeZone: window.chartTimeZone, day: 'numeric', month: 'short', year: '2-digit', hour: '2-digit', minute: '2-digit', hour12: false }).replace(',', '') };
        const timeScale = { tickMarkFormatter: (time) => { const d = typeof time === 'number' ? new Date(time * 1000) : new Date(time); return isNaN(d) ? "" : d.toLocaleDateString('en-GB', { month: 'short', day: 'numeric', timeZone: window.chartTimeZone }); } };
        window.charts.forEach(c => c.applyOptions({ localization, timeScale }));
    },
    set_timeframe: (_targetChart, cmd) => {
        // Timeframe display removed from status indicator
    },
    hide_loading: () => {
        if (window.hideLoader) window.hideLoader();
    },
    show_notification: (_targetChart, cmd) => {
        // Ensure notification container exists and has correct styling
        let container = document.getElementById('notification-container');
        if (!container) {
            container = document.createElement('div');
            container.id = 'notification-container';
            container.style.cssText = `position: fixed; bottom: 20px; left: 20px; z-index: 10000; display: flex; flex-direction: column-reverse; gap: 10px; pointer-events: none;`;
            document.body.appendChild(container);
        }
        showNotification(cmd.data.message, cmd.data.type || 'info', cmd.data.duration || 3000, cmd.data.text_color || null);
    },
    set_trend_info_visibility: (_targetChart, cmd) => {
        const panel = document.getElementById('trend-info');
        if (panel) panel.classList.toggle('hidden', !cmd.data.visible);
    },
    set_layout_toolbar_visibility: (_targetChart, cmd) => {
        const toolbar = document.getElementById('toolbar');
        if (toolbar) toolbar.classList.toggle('hidden', !cmd.data.visible);
    },
    set_legend_visibility: (_targetChart, cmd) => {
        const legend = document.getElementById('legend');
        if (legend) legend.classList.toggle('hidden', !cmd.data.visible);
    },
    update_positions: (_targetChart, cmd) => {
        if (window.updatePositionsUI) window.updatePositionsUI(cmd.data);
    },
    set_trading_visibility: (_targetChart, cmd) => {
        const trade = document.getElementById('trade-panel');
        const pos = document.getElementById('positions-panel');
        const visible = cmd.data.visible;
        if (trade) {
            trade.classList.toggle('hidden', !visible);
        }
        if (pos) {
            pos.classList.toggle('hidden', !visible);
            window.positionsUserHidden = !visible;
        }
    },
    update_trend: (_targetChart, cmd) => {
        const d = cmd.data || {};
        const content = document.getElementById('trend-content');
        if (!content) return;

        Object.entries(d).forEach(([key, val]) => {
            const elId = `trend-${key.toLowerCase()}`;
            let row = document.getElementById(elId);
            if (!row) {
                row = document.createElement('div');
                row.className = 'trend-row';
                row.id = elId;
                row.innerHTML = `<span>${key.toUpperCase()}</span><span class="trend-val">NA</span>`;
                content.appendChild(row);
            }
            const valEl = row.querySelector('.trend-val');
            if (valEl) {
                valEl.textContent = val || 'NA';
                valEl.className = `trend-val trend-${String(val).toLowerCase() || 'neutral'}`;
            }
        });

        const panel = document.getElementById('trend-info');
        if (panel) {
            if (panel.style.display === 'none') panel.style.display = '';
            panel.classList.remove('collapsed');
        }
    },
    take_screenshot: (targetChart, _cmd, chartId) => {
        if (targetChart) {
            const canvas = targetChart.takeScreenshot();
            const a = document.createElement('a');
            a.href = canvas.toDataURL();
            a.download = `chart_${chartId}_${new Date().toISOString()}.png`;
            a.click();
        }
    },
    // Aliases for Rust backend consistency
    set_series_data: (targetChart, cmd) => CommandHandlers.set_data(targetChart, cmd),
    update_series_data: (targetChart, cmd) => CommandHandlers.update_data(targetChart, cmd)
};

CommandQueue.processCommandSync = CommandQueue.processCommandSync.bind(CommandQueue);
// window.CommandHandlers = CommandHandlers; 
// window.handleCommand = handleCommand;
// window.hideLoader = hideLoader;

// Final Initialization moved to entry.js
